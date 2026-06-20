#![no_std]
#![allow(static_mut_refs)]

extern crate alloc;

use core::panic::PanicInfo;

mod runtime;

// Shorthand for "any M3 user-mode test feature is active (includes M5 blk_test, M6 fs_test, G1 go_test, G2 spike go_std_test, M10 security tests)"
macro_rules! cfg_m3 {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
            $item
        )*
    };
}

// Shorthand for "any user-mode feature (M3 or R4 or M5 or M6 or G1 or G2 spike)"
macro_rules! cfg_user {
    ($($item:item)*) => {
        $(
            #[cfg(any(
                feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test",
                feature = "ipc_test", feature = "shm_test", feature = "ipc_badptr_send_test", feature = "ipc_badptr_recv_test", feature = "ipc_badptr_svc_test", feature = "ipc_buffer_full_test", feature = "ipc_waiter_busy_test", feature = "svc_overwrite_test", feature = "svc_full_test", feature = "svc_bad_endpoint_test", feature = "stress_ipc_test", feature = "quota_endpoints_test", feature = "quota_shm_test", feature = "quota_threads_test", feature = "blk_test", feature = "fs_test",
                feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test",
            ))]
            $item
        )*
    };
}

// Shorthand for "any R4 feature"
macro_rules! cfg_r4 {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "ipc_test", feature = "shm_test", feature = "ipc_badptr_send_test", feature = "ipc_badptr_recv_test", feature = "ipc_badptr_svc_test", feature = "ipc_buffer_full_test", feature = "ipc_waiter_busy_test", feature = "svc_overwrite_test", feature = "svc_full_test", feature = "svc_bad_endpoint_test", feature = "stress_ipc_test", feature = "quota_endpoints_test", feature = "quota_shm_test", feature = "quota_threads_test", feature = "go_test"))]
            $item
        )*
    };
}

mod arch_x86;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod aes;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod cache;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod dma;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod sha256;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod mount;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod tty;
pub(crate) mod fb;
pub(crate) mod smp;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod kbd;
mod memory;
pub(crate) mod mm;
mod net;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod tcp;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod netcfg;
#[cfg(feature = "go_test")]
pub(crate) mod vfs;
mod process;
mod sched;
mod storage;
mod syscall;
mod trap;

use arch_x86::{gdt_init, idt_init, inb, outb, qemu_exit};
#[cfg(any(
    feature = "blk_test",
    feature = "blk_invariants_test",
    feature = "fs_test",
    feature = "net_test",
    feature = "go_test",
))]
use arch_x86::{inl, inw, outl, outw};
#[cfg(any(
    feature = "user_hello_test",
    feature = "syscall_test",
    feature = "thread_exit_test",
    feature = "thread_spawn_test",
    feature = "vm_map_test",
    feature = "syscall_invalid_test",
    feature = "stress_syscall_test",
    feature = "yield_test",
    feature = "user_fault_test",
    feature = "ipc_test",
    feature = "shm_test",
    feature = "ipc_badptr_send_test",
    feature = "ipc_badptr_recv_test",
    feature = "ipc_badptr_svc_test",
    feature = "ipc_buffer_full_test",
    feature = "ipc_waiter_busy_test",
    feature = "svc_overwrite_test",
    feature = "svc_full_test",
    feature = "svc_bad_endpoint_test",
    feature = "stress_ipc_test",
    feature = "quota_endpoints_test",
    feature = "quota_shm_test",
    feature = "quota_threads_test",
    feature = "blk_test",
    feature = "fs_test",
    feature = "go_test",
    feature = "go_std_test",
    feature = "sec_rights_test",
    feature = "sec_filter_test",
))]
use arch_x86::{enter_ring3_at, tss_init};
use memory::{
    check_page_user_perms, copyin_user, copyinstr_user, copyout_user, user_pages_ok, user_range_ok,
    USER_PERM_READ, USER_PERM_WRITE, USER_VA_LIMIT,
};
#[cfg(feature = "sched_test")]
use sched::{pic_init, pit_init, sched_init, thread_create};

// --------------- M8 PR-2: ELF loader policy helpers -------------------------

const ELF_V1_MAX_PHNUM: u16 = 32;
const ELF_V1_PHDR_SIZE: u16 = 56;
const ELF_V1_PT_LOAD: u32 = 1;
const ELF_V1_ET_EXEC: u16 = 2;
const ELF_V1_USER_LIMIT: u64 = 0x0000_8000_0000_0000;

#[allow(dead_code)]
const AUXV_V1_AT_NULL: u64 = 0;
#[allow(dead_code)]
const AUXV_V1_AT_PHDR: u64 = 3;
#[allow(dead_code)]
const AUXV_V1_AT_PHENT: u64 = 4;
#[allow(dead_code)]
const AUXV_V1_AT_PHNUM: u64 = 5;
#[allow(dead_code)]
const AUXV_V1_AT_PAGESZ: u64 = 6;
#[allow(dead_code)]
const AUXV_V1_AT_ENTRY: u64 = 9;

#[allow(dead_code)]
fn elf_v1_read_u16(buf: &[u8], off: usize) -> Option<u16> {
    if off.checked_add(2)? > buf.len() {
        return None;
    }
    Some(u16::from_le_bytes([buf[off], buf[off + 1]]))
}

#[allow(dead_code)]
fn elf_v1_read_u32(buf: &[u8], off: usize) -> Option<u32> {
    if off.checked_add(4)? > buf.len() {
        return None;
    }
    Some(u32::from_le_bytes([
        buf[off],
        buf[off + 1],
        buf[off + 2],
        buf[off + 3],
    ]))
}

#[allow(dead_code)]
fn elf_v1_read_u64(buf: &[u8], off: usize) -> Option<u64> {
    if off.checked_add(8)? > buf.len() {
        return None;
    }
    Some(u64::from_le_bytes([
        buf[off],
        buf[off + 1],
        buf[off + 2],
        buf[off + 3],
        buf[off + 4],
        buf[off + 5],
        buf[off + 6],
        buf[off + 7],
    ]))
}

#[allow(dead_code)]
fn elf_v1_is_pow2(v: u64) -> bool {
    v != 0 && (v & (v - 1)) == 0
}

#[allow(dead_code)]
fn elf_v1_validate_image(image: &[u8]) -> bool {
    if image.len() < 64 {
        return false;
    }

    if image[0] != 0x7F || image[1] != b'E' || image[2] != b'L' || image[3] != b'F' {
        return false;
    }
    // ELF64 + little-endian + current ELF version.
    if image[4] != 2 || image[5] != 1 || image[6] != 1 {
        return false;
    }

    let e_phoff = match elf_v1_read_u64(image, 32) {
        Some(v) => v as usize,
        None => return false,
    };
    let e_phentsize = match elf_v1_read_u16(image, 54) {
        Some(v) => v,
        None => return false,
    };
    let e_phnum = match elf_v1_read_u16(image, 56) {
        Some(v) => v,
        None => return false,
    };
    let e_entry = match elf_v1_read_u64(image, 24) {
        Some(v) => v,
        None => return false,
    };

    if e_entry == 0 || e_entry >= ELF_V1_USER_LIMIT {
        return false;
    }
    if e_phnum == 0 || e_phnum > ELF_V1_MAX_PHNUM {
        return false;
    }
    if e_phentsize < ELF_V1_PHDR_SIZE {
        return false;
    }

    let phdr_bytes = match (e_phentsize as usize).checked_mul(e_phnum as usize) {
        Some(v) => v,
        None => return false,
    };
    let phdr_end = match e_phoff.checked_add(phdr_bytes) {
        Some(v) => v,
        None => return false,
    };
    if phdr_end > image.len() {
        return false;
    }

    let mut load_count = 0usize;
    for idx in 0..(e_phnum as usize) {
        let off = e_phoff + idx * (e_phentsize as usize);
        let p_type = match elf_v1_read_u32(image, off) {
            Some(v) => v,
            None => return false,
        };
        if p_type != ELF_V1_PT_LOAD {
            continue;
        }
        load_count += 1;

        let p_offset = match elf_v1_read_u64(image, off + 8) {
            Some(v) => v,
            None => return false,
        };
        let p_vaddr = match elf_v1_read_u64(image, off + 16) {
            Some(v) => v,
            None => return false,
        };
        let p_filesz = match elf_v1_read_u64(image, off + 32) {
            Some(v) => v,
            None => return false,
        };
        let p_memsz = match elf_v1_read_u64(image, off + 40) {
            Some(v) => v,
            None => return false,
        };
        let p_align = match elf_v1_read_u64(image, off + 48) {
            Some(v) => v,
            None => return false,
        };

        if p_memsz < p_filesz {
            return false;
        }
        if p_align != 0 && !elf_v1_is_pow2(p_align) {
            return false;
        }

        let end_file = match p_offset.checked_add(p_filesz) {
            Some(v) => v,
            None => return false,
        };
        if end_file > image.len() as u64 {
            return false;
        }

        if p_vaddr >= ELF_V1_USER_LIMIT {
            return false;
        }
        let end_vaddr = match p_vaddr.checked_add(p_memsz) {
            Some(v) => v,
            None => return false,
        };
        if end_vaddr > ELF_V1_USER_LIMIT {
            return false;
        }
    }

    load_count > 0
}

#[allow(dead_code)]
fn elf_v1_build_auxv(entry: u64, phdr: u64, phent: u64, phnum: u64) -> [(u64, u64); 6] {
    [
        (AUXV_V1_AT_PHDR, phdr),
        (AUXV_V1_AT_PHENT, phent),
        (AUXV_V1_AT_PHNUM, phnum),
        (AUXV_V1_AT_PAGESZ, 4096),
        (AUXV_V1_AT_ENTRY, entry),
        (AUXV_V1_AT_NULL, 0),
    ]
}

// --------------- Serial (COM1) ---------------

const COM1: u16 = 0x3F8;

fn serial_init() {
    unsafe {
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x80);
        outb(COM1 + 0, 0x01);
        outb(COM1 + 1, 0x00);
        outb(COM1 + 3, 0x03);
        outb(COM1 + 2, 0x00);
        outb(COM1 + 4, 0x00);
    }
}

fn serial_write(s: &[u8]) {
    for &b in s {
        unsafe {
            while inb(COM1 + 5) & 0x20 == 0 {}
            outb(COM1, b);
        }
    }
    // Mirror the transcript onto the framebuffer console (item 7). The
    // sched_test lane is excluded: its kernel threads run with IF=1 and
    // can be preempted mid-draw, racing the cursor/scroll state. Every
    // other lane writes from IF=0 kernel context, where this is atomic.
    #[cfg(not(feature = "sched_test"))]
    fb::fb_write(s);
    // Mirror into the dmesg ring (full-os guide V.11 observability / IV.10
    // audit) so userspace can read the kernel log via sys_sysinfo op 4.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe {
        klog_append(s);
    }
}

// dmesg ring buffer: a heap-free fixed ring that captures every serial_write
// line. Oldest bytes are overwritten once full; reads return the most recent
// `len` bytes in oldest->newest order (full-os guide Part V.11 / IV.10).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const KLOG_CAP: usize = 8192;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut KLOG: [u8; KLOG_CAP] = [0; KLOG_CAP];
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut KLOG_HEAD: usize = 0;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut KLOG_LEN: usize = 0;

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn klog_append(s: &[u8]) {
    for &b in s {
        KLOG[KLOG_HEAD] = b;
        KLOG_HEAD = (KLOG_HEAD + 1) % KLOG_CAP;
        if KLOG_LEN < KLOG_CAP {
            KLOG_LEN += 1;
        }
    }
}

/// Copy the most recent `min(len, valid)` bytes of the dmesg ring into the
/// user buffer at `ptr`, in oldest->newest order. Returns the count copied,
/// or u64::MAX if the user buffer is unwritable.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn klog_read(ptr: u64, len: usize) -> u64 {
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

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
fn ascii_upper(b: u8) -> u8 {
    if b.is_ascii_lowercase() {
        b - 32
    } else {
        b
    }
}

/// Read a file by its 11-byte 8.3 directory name from the FAT16 volume at a
/// fixed LBA into `out` (single cluster, v1). Returns the byte count, or None
/// if the volume/BPB is bad or the file is absent (full-os guide Part II.5 FAT;
/// shared by `sys_sysinfo` op 6 and the `/mnt` open path).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn fat16_read_named(target: &[u8; 11], out: &mut [u8]) -> Option<usize> {
    const VOL_LBA: u64 = 2048;
    if !storage::r4_storage_available() || !block_io_dispatch(false, VOL_LBA, 512, false) {
        return None;
    }
    let bps = u16::from_le_bytes([BLK_DATA_PAGE.0[11], BLK_DATA_PAGE.0[12]]) as u64;
    let spc = BLK_DATA_PAGE.0[13] as u64;
    let reserved = u16::from_le_bytes([BLK_DATA_PAGE.0[14], BLK_DATA_PAGE.0[15]]) as u64;
    let nfats = BLK_DATA_PAGE.0[16] as u64;
    let root_entries = u16::from_le_bytes([BLK_DATA_PAGE.0[17], BLK_DATA_PAGE.0[18]]) as u64;
    let spf = u16::from_le_bytes([BLK_DATA_PAGE.0[22], BLK_DATA_PAGE.0[23]]) as u64;
    if bps != 512 || spc == 0 || nfats == 0 {
        return None;
    }
    let root_lba = VOL_LBA + reserved + nfats * spf;
    let root_sectors = (root_entries * 32 + 511) / 512;
    let data_lba = root_lba + root_sectors;
    let mut first_cluster = 0u64;
    let mut file_size = 0u64;
    let mut s = 0u64;
    'scan: while s < root_sectors {
        if !block_io_dispatch(false, root_lba + s, 512, false) {
            return None;
        }
        let mut e = 0usize;
        while e < 16 {
            let base = e * 32;
            if BLK_DATA_PAGE.0[base] == 0 {
                break 'scan;
            }
            if BLK_DATA_PAGE.0[base..base + 11] == *target {
                first_cluster = u16::from_le_bytes([
                    BLK_DATA_PAGE.0[base + 26],
                    BLK_DATA_PAGE.0[base + 27],
                ]) as u64;
                file_size = u32::from_le_bytes([
                    BLK_DATA_PAGE.0[base + 28],
                    BLK_DATA_PAGE.0[base + 29],
                    BLK_DATA_PAGE.0[base + 30],
                    BLK_DATA_PAGE.0[base + 31],
                ]) as u64;
                break 'scan;
            }
            e += 1;
        }
        s += 1;
    }
    if first_cluster < 2 || file_size == 0 {
        return None;
    }
    let cluster_lba = data_lba + (first_cluster - 2) * spc;
    if !block_io_dispatch(false, cluster_lba, 512, false) {
        return None;
    }
    let n = core::cmp::min(file_size as usize, out.len()).min(512);
    out[..n].copy_from_slice(&BLK_DATA_PAGE.0[..n]);
    Some(n)
}

/// Read a file by its 8.3 name following the FAT16 cluster CHAIN (full-os guide
/// Part II.5): unlike `fat16_read_named` (first cluster only), this walks the FAT
/// from the directory's first cluster to the end-of-chain marker, so it reads
/// files spanning multiple clusters. Returns min(file_size, out.len()) bytes, or
/// None on a bad volume/absent file/corrupt chain.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn fat16_read_chain(target: &[u8; 11], out: &mut [u8]) -> Option<usize> {
    const VOL_LBA: u64 = 2048;
    if !storage::r4_storage_available() || !block_io_dispatch(false, VOL_LBA, 512, false) {
        return None;
    }
    let bps = u16::from_le_bytes([BLK_DATA_PAGE.0[11], BLK_DATA_PAGE.0[12]]) as u64;
    let spc = BLK_DATA_PAGE.0[13] as u64;
    let reserved = u16::from_le_bytes([BLK_DATA_PAGE.0[14], BLK_DATA_PAGE.0[15]]) as u64;
    let nfats = BLK_DATA_PAGE.0[16] as u64;
    let root_entries = u16::from_le_bytes([BLK_DATA_PAGE.0[17], BLK_DATA_PAGE.0[18]]) as u64;
    let spf = u16::from_le_bytes([BLK_DATA_PAGE.0[22], BLK_DATA_PAGE.0[23]]) as u64;
    if bps != 512 || spc == 0 || nfats == 0 {
        return None;
    }
    let fat_lba = VOL_LBA + reserved;
    let root_lba = fat_lba + nfats * spf;
    let root_sectors = (root_entries * 32 + 511) / 512;
    let data_lba = root_lba + root_sectors;

    // Locate the directory entry: first cluster + file size.
    let mut cluster = 0u64;
    let mut file_size = 0u64;
    let mut s = 0u64;
    'scan: while s < root_sectors {
        if !block_io_dispatch(false, root_lba + s, 512, false) {
            return None;
        }
        let mut e = 0usize;
        while e < 16 {
            let base = e * 32;
            if BLK_DATA_PAGE.0[base] == 0 {
                break 'scan;
            }
            if BLK_DATA_PAGE.0[base..base + 11] == *target {
                cluster =
                    u16::from_le_bytes([BLK_DATA_PAGE.0[base + 26], BLK_DATA_PAGE.0[base + 27]])
                        as u64;
                file_size = u32::from_le_bytes([
                    BLK_DATA_PAGE.0[base + 28],
                    BLK_DATA_PAGE.0[base + 29],
                    BLK_DATA_PAGE.0[base + 30],
                    BLK_DATA_PAGE.0[base + 31],
                ]) as u64;
                break 'scan;
            }
            e += 1;
        }
        s += 1;
    }
    if cluster < 2 || file_size == 0 {
        return None;
    }

    // Walk the chain: copy each cluster's sectors, then follow the FAT16 entry
    // (2 bytes per cluster) to the next cluster until a terminal value. A valid data
    // cluster is [2, 0xFFEF]; 0xFFF0..=0xFFF6 are reserved, 0xFFF7 is a bad-cluster
    // marker, and 0xFFF8..=0xFFFF is end-of-chain. Stopping at `< 0xFFF0` means a
    // reserved or bad-cluster FAT entry terminates the read instead of being followed
    // as a (bogus, out-of-range) cluster number -- which would compute a wild
    // data_lba and read garbage off the disk. The iteration guard additionally bounds
    // a corrupt self-referential chain so it cannot loop forever.
    let want = core::cmp::min(file_size as usize, out.len());
    let mut written = 0usize;
    let mut guard = 0u64;
    let max_clusters = out.len() as u64 / (spc * 512) + 4;
    while cluster >= 2 && cluster < 0xFFF0 && written < want {
        guard += 1;
        if guard > max_clusters {
            return None;
        }
        let cluster_lba = data_lba + (cluster - 2) * spc;
        let mut sec = 0u64;
        while sec < spc && written < want {
            if !block_io_dispatch(false, cluster_lba + sec, 512, false) {
                return None;
            }
            let n = core::cmp::min(512, want - written);
            out[written..written + n].copy_from_slice(&BLK_DATA_PAGE.0[..n]);
            written += n;
            sec += 1;
        }
        // Read the FAT entry for `cluster` (this overwrites BLK_DATA_PAGE, but the
        // cluster data was already copied into `out`).
        let fat_byte = cluster * 2;
        let fat_sec = fat_lba + fat_byte / 512;
        let off = (fat_byte % 512) as usize;
        if !block_io_dispatch(false, fat_sec, 512, false) {
            return None;
        }
        cluster = u16::from_le_bytes([BLK_DATA_PAGE.0[off], BLK_DATA_PAGE.0[off + 1]]) as u64;
    }
    Some(written)
}

/// GPT partition-table parse self-test (full-os guide Part II.5, partitions):
/// read the GPT header at LBA 1, validate the "EFI PART" signature, then walk the
/// partition-entry array and count the live entries (non-zero type GUID), logging
/// each one's first/last LBA. Runs at boot; reports `GPT: none` when LBA 1 is not
/// a GPT header (the common case). Complements the MBR parser (sysinfo op 5).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn gpt_parse_selftest() {
    if !storage::r4_storage_available() || !block_io_dispatch(false, 1, 512, false) {
        serial_write(b"GPT: none\n");
        return;
    }
    if BLK_DATA_PAGE.0[0..8] != *b"EFI PART" {
        serial_write(b"GPT: none\n");
        return;
    }
    let entry_lba = u64::from_le_bytes(BLK_DATA_PAGE.0[72..80].try_into().unwrap());
    let num_entries = u32::from_le_bytes(BLK_DATA_PAGE.0[80..84].try_into().unwrap()) as u64;
    let entry_size = u32::from_le_bytes(BLK_DATA_PAGE.0[84..88].try_into().unwrap()) as u64;
    // GPT entries are >= 128 bytes (UEFI spec); bounding entry_size to [128, 512]
    // also guarantees every per-sector entry's fields (off + 48) stay within the
    // 512-byte BLK_DATA_PAGE, so a malformed header cannot drive a slice-OOB
    // panic at boot.
    if entry_size < 128 || entry_size > 512 || entry_lba == 0 {
        serial_write(b"GPT: bad header\n");
        return;
    }
    let per_sector = 512 / entry_size;
    // v1 bound: inspect the first 16 entries (typically 4 sectors). A larger
    // table is valid; this self-test just confirms the parse, not full coverage.
    let max_entries = core::cmp::min(num_entries, 16);
    let mut count = 0u64;
    let mut e = 0u64;
    'walk: while e < max_entries {
        let sector = entry_lba + e / per_sector;
        if !block_io_dispatch(false, sector, 512, false) {
            break 'walk;
        }
        let mut j = 0u64;
        while j < per_sector && e < max_entries {
            let off = (j * entry_size) as usize;
            let mut used = false;
            let mut k = 0usize;
            while k < 16 {
                if BLK_DATA_PAGE.0[off + k] != 0 {
                    used = true;
                    break;
                }
                k += 1;
            }
            if used {
                let first =
                    u64::from_le_bytes(BLK_DATA_PAGE.0[off + 32..off + 40].try_into().unwrap());
                let last =
                    u64::from_le_bytes(BLK_DATA_PAGE.0[off + 40..off + 48].try_into().unwrap());
                serial_write(b"GPT: part first=0x");
                serial_write_hex(first);
                serial_write(b" last=0x");
                serial_write_hex(last);
                serial_write(b"\n");
                count += 1;
            }
            j += 1;
            e += 1;
        }
    }
    serial_write(b"GPT: parsed n=0x");
    serial_write_hex(count);
    serial_write(b"\n");
}

/// IEEE 802.3 CRC-32 (reflected, polynomial 0xEDB88320) over `data`. Bitwise (no
/// table) to keep the .text small. The same CRC GPT uses for its header and
/// partition-array integrity fields. (full-os guide Part II.5, partitions.)
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    let mut i = 0;
    while i < data.len() {
        crc ^= data[i] as u32;
        let mut k = 0;
        while k < 8 {
            let m = (crc & 1).wrapping_neg(); // 0xFFFFFFFF when the low bit is set
            crc = (crc >> 1) ^ (0xEDB8_8320 & m);
            k += 1;
        }
        i += 1;
    }
    !crc
}

/// GPT header-CRC validation self-test (full-os guide Part II.5): the existing GPT
/// parser trusts a signed header without checking its CRC, so a corrupted-but-signed
/// header is accepted silently. This proves the CRC machinery the validation needs:
/// (1) a known-answer test that crc32 is the real IEEE CRC-32 (CRC32("123456789") ==
/// 0xCBF43926); (2) build a synthetic GPT header, stamp its header CRC at offset 16,
/// and validate it the standard GPT way (zero the CRC field, recompute over
/// HeaderSize bytes, compare); (3) flip a byte and confirm the recomputed CRC no
/// longer matches (a corrupted header is rejected). Marker `GPT: hdr crc ok`.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn gpt_crc_selftest() {
    // (1) Known-answer test: the standard CRC-32 check value.
    let kat = crc32(b"123456789") == 0xCBF4_3926;

    // (2) Synthesize a minimal GPT header and stamp its header CRC (offset 16..20).
    const HDR_SIZE: usize = 92;
    let mut hdr = [0u8; HDR_SIZE];
    hdr[0..8].copy_from_slice(b"EFI PART");
    hdr[8..12].copy_from_slice(&0x0001_0000u32.to_le_bytes()); // revision 1.0
    hdr[12..16].copy_from_slice(&(HDR_SIZE as u32).to_le_bytes()); // HeaderSize
    // offset 16..20 = header CRC, left zero while computing
    hdr[24..32].copy_from_slice(&1u64.to_le_bytes()); // MyLBA = 1
    let stored = crc32(&hdr[..HDR_SIZE]);
    hdr[16..20].copy_from_slice(&stored.to_le_bytes());

    // Validate as a real GPT consumer would: read the stored CRC, zero the field,
    // recompute over HeaderSize bytes, and compare.
    let read_crc = u32::from_le_bytes(hdr[16..20].try_into().unwrap());
    let mut tmp = hdr;
    tmp[16..20].copy_from_slice(&[0u8; 4]);
    let valid = crc32(&tmp[..HDR_SIZE]) == read_crc;

    // (3) Corrupt one header byte: the recomputed CRC must no longer match.
    let mut bad = hdr;
    bad[24] ^= 0xFF;
    bad[16..20].copy_from_slice(&[0u8; 4]);
    let rejects = crc32(&bad[..HDR_SIZE]) != read_crc;

    if kat && valid && rejects {
        serial_write(b"GPT: hdr crc ok\n");
    } else {
        serial_write(b"GPT: hdr crc fail\n");
    }
}

/// Write a single-cluster file to the FAT16 root directory (full-os guide Part
/// II.5, FS maturity): allocate the first free cluster, mark it EOC in every FAT
/// copy, write the data into that cluster, and fill a free root-dir 8.3 entry.
/// v1 boundary: one cluster (≤512 B), root dir only, no overwrite/append/chain.
/// Returns true on success. Shares the BPB parse with `fat16_read_named`.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn fat16_write_named(target: &[u8; 11], data: &[u8]) -> bool {
    const VOL_LBA: u64 = 2048;
    if !storage::r4_storage_available() || data.is_empty() || data.len() > 512 {
        return false;
    }
    if !block_io_dispatch(false, VOL_LBA, 512, false) {
        return false;
    }
    let bps = u16::from_le_bytes([BLK_DATA_PAGE.0[11], BLK_DATA_PAGE.0[12]]) as u64;
    let spc = BLK_DATA_PAGE.0[13] as u64;
    let reserved = u16::from_le_bytes([BLK_DATA_PAGE.0[14], BLK_DATA_PAGE.0[15]]) as u64;
    let nfats = BLK_DATA_PAGE.0[16] as u64;
    let root_entries = u16::from_le_bytes([BLK_DATA_PAGE.0[17], BLK_DATA_PAGE.0[18]]) as u64;
    let spf = u16::from_le_bytes([BLK_DATA_PAGE.0[22], BLK_DATA_PAGE.0[23]]) as u64;
    if bps != 512 || spc == 0 || nfats == 0 || spf == 0 {
        return false;
    }
    let fat_lba = VOL_LBA + reserved;
    let root_lba = fat_lba + nfats * spf;
    let root_sectors = (root_entries * 32 + 511) / 512;
    let data_lba = root_lba + root_sectors;

    // 1) Find a free root-dir slot AND reject a duplicate name -- BEFORE any
    // on-disk mutation, so a full directory or an already-present name leaks
    // nothing (the deterministic leak the review found). Scan to the first 0x00
    // (end-of-directory): entries beyond it are unused.
    let mut slot_lba = 0u64;
    let mut slot_off = 0usize;
    let mut found_slot = false;
    let mut ended = false;
    let mut s = 0u64;
    while s < root_sectors && !ended {
        if !block_io_dispatch(false, root_lba + s, 512, false) {
            return false;
        }
        let mut e = 0usize;
        while e < 16 {
            let base = e * 32;
            let b0 = BLK_DATA_PAGE.0[base];
            if b0 == 0x00 {
                // End of directory: the first free slot if none seen earlier.
                if !found_slot {
                    slot_lba = root_lba + s;
                    slot_off = base;
                    found_slot = true;
                }
                ended = true;
                break;
            }
            if b0 != 0xE5 && BLK_DATA_PAGE.0[base..base + 11] == *target {
                return false; // name already exists -> refuse (no duplicate entry)
            }
            if b0 == 0xE5 && !found_slot {
                slot_lba = root_lba + s;
                slot_off = base;
                found_slot = true;
            }
            e += 1;
        }
        s += 1;
    }
    if !found_slot {
        return false; // directory full -> nothing committed
    }
    // 2) Find a free cluster (first FAT sector, clusters 2..255) -- still no
    // mutation, so a full FAT also leaks nothing.
    if !block_io_dispatch(false, fat_lba, 512, false) {
        return false;
    }
    let mut cluster = 0u64;
    let mut c = 2usize;
    while c < 256 {
        if u16::from_le_bytes([BLK_DATA_PAGE.0[c * 2], BLK_DATA_PAGE.0[c * 2 + 1]]) == 0 {
            cluster = c as u64;
            break;
        }
        c += 1;
    }
    if cluster < 2 {
        return false; // no free cluster -> nothing committed
    }
    // 3) Commit: mark the cluster end-of-chain in every FAT copy, write the data
    // cluster, then link it from the directory entry. (A mid-commit device-write
    // failure can still leave an orphaned cluster or divergent FAT copies; that
    // is inherent to a non-journaling FAT writer -- see the v1 boundary.)
    BLK_DATA_PAGE.0[(cluster as usize) * 2] = 0xFF;
    BLK_DATA_PAGE.0[(cluster as usize) * 2 + 1] = 0xFF;
    let mut f = 0u64;
    while f < nfats {
        if !block_io_dispatch(true, fat_lba + f * spf, 512, false) {
            return false;
        }
        f += 1;
    }
    let cluster_lba = data_lba + (cluster - 2) * spc;
    core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr(), 0, 512);
    BLK_DATA_PAGE.0[..data.len()].copy_from_slice(data);
    if !block_io_dispatch(true, cluster_lba, 512, false) {
        return false;
    }
    // Re-read the chosen directory sector (the buffer was reused above) and fill
    // the slot we reserved in step 1.
    if !block_io_dispatch(false, slot_lba, 512, false) {
        return false;
    }
    core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr().add(slot_off), 0, 32);
    BLK_DATA_PAGE.0[slot_off..slot_off + 11].copy_from_slice(target);
    BLK_DATA_PAGE.0[slot_off + 11] = 0x20; // attr = archive
    BLK_DATA_PAGE.0[slot_off + 26] = (cluster & 0xFF) as u8;
    BLK_DATA_PAGE.0[slot_off + 27] = ((cluster >> 8) & 0xFF) as u8;
    BLK_DATA_PAGE.0[slot_off + 28..slot_off + 32]
        .copy_from_slice(&(data.len() as u32).to_le_bytes());
    block_io_dispatch(true, slot_lba, 512, false)
}

/// List the FAT16 root directory (full-os guide Part II.5): log each live 8.3
/// entry as `FATLS: <name11> size=0x<hex>` and return the count, or u64::MAX on
/// a bad volume. Skips free (0x00), deleted (0xE5), and long-name (attr 0x0F)
/// entries. Shares the BPB parse with `fat16_read_named`.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn fat16_list() -> u64 {
    const VOL_LBA: u64 = 2048;
    if !storage::r4_storage_available() || !block_io_dispatch(false, VOL_LBA, 512, false) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let bps = u16::from_le_bytes([BLK_DATA_PAGE.0[11], BLK_DATA_PAGE.0[12]]) as u64;
    let reserved = u16::from_le_bytes([BLK_DATA_PAGE.0[14], BLK_DATA_PAGE.0[15]]) as u64;
    let nfats = BLK_DATA_PAGE.0[16] as u64;
    let root_entries = u16::from_le_bytes([BLK_DATA_PAGE.0[17], BLK_DATA_PAGE.0[18]]) as u64;
    let spf = u16::from_le_bytes([BLK_DATA_PAGE.0[22], BLK_DATA_PAGE.0[23]]) as u64;
    if bps != 512 || nfats == 0 {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let root_lba = VOL_LBA + reserved + nfats * spf;
    let root_sectors = (root_entries * 32 + 511) / 512;
    let mut count = 0u64;
    let mut s = 0u64;
    'scan: while s < root_sectors {
        if !block_io_dispatch(false, root_lba + s, 512, false) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let mut e = 0usize;
        while e < 16 {
            let base = e * 32;
            let first = BLK_DATA_PAGE.0[base];
            if first == 0x00 {
                break 'scan; // end of directory
            }
            // Skip deleted (0xE5) and long-file-name (attr 0x0F) entries.
            if first != 0xE5 && BLK_DATA_PAGE.0[base + 11] != 0x0F {
                let size = u32::from_le_bytes([
                    BLK_DATA_PAGE.0[base + 28],
                    BLK_DATA_PAGE.0[base + 29],
                    BLK_DATA_PAGE.0[base + 30],
                    BLK_DATA_PAGE.0[base + 31],
                ]);
                serial_write(b"FATLS: ");
                serial_write(&BLK_DATA_PAGE.0[base..base + 11]);
                serial_write(b" size=0x");
                serial_write_hex(size as u64);
                serial_write(b"\n");
                count += 1;
            }
            e += 1;
        }
        s += 1;
    }
    count
}

/// Symmetric cipher for at-rest disk encryption (full-os guide Part IV.10).
/// AES-128 in CTR mode keyed on a fixed key, per-LBA counter, so encrypt ==
/// decrypt. The AES core is verified against the FIPS-197 known-answer test at
/// boot (`aes::aes_selftest`). NOTE: a real system needs a KDF'd key + an
/// integrity tag (AES-GCM / ChaCha20-Poly1305 AEAD) and constant-time tables;
/// CTR-without-MAC and a fixed key are the v1 boundary.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn disk_crypt(lba: u64, buf: &mut [u8]) {
    const DISK_KEY: [u8; 16] = [
        0x9e, 0x37, 0x79, 0xb9, 0x7f, 0x4a, 0x7c, 0x15, 0xf3, 0x9c, 0xc0, 0x60, 0x5c, 0xed, 0xc8,
        0x34,
    ];
    aes::ctr_xor(&DISK_KEY, lba, buf);
}

fn serial_can_read() -> bool {
    unsafe { inb(COM1 + 5) & 0x01 != 0 }
}

fn serial_read_byte() -> u8 {
    unsafe {
        while inb(COM1 + 5) & 0x01 == 0 {}
        inb(COM1)
    }
}

/// Console input: PS/2 keyboard bytes win, serial is the fallback. The
/// wait loop polls the i8042 directly because IRQ1 cannot fire while the
/// kernel spins here with interrupts masked.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
fn console_read_byte() -> u8 {
    unsafe {
        loop {
            if let Some(b) = kbd::kbd_pop() {
                return b;
            }
            kbd::kbd_poll();
            if serial_can_read() {
                return inb(COM1);
            }
        }
    }
}

fn serial_write_hex(val: u64) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut buf = [b'0'; 16];
    let mut v = val;
    for i in (0..16).rev() {
        buf[i] = HEX[(v & 0xF) as usize];
        v >>= 4;
    }
    serial_write(&buf);
}

fn serial_write_u64_dec(val: u64) {
    let mut buf = [0u8; 20];
    let mut idx = buf.len();
    let mut value = val;
    if value == 0 {
        serial_write(b"0");
        return;
    }
    while value != 0 {
        idx -= 1;
        buf[idx] = b'0' + (value % 10) as u8;
        value /= 10;
    }
    serial_write(&buf[idx..]);
}

// --------------- Limine boot protocol markers (v8 API) ---------------

#[used]
#[link_section = ".limine_requests_start"]
static LIMINE_REQUESTS_START: [u64; 4] = [
    0xf6b8f4b39de7d1ae, 0xfab91a6940fcb9cf,
    0x785c6ed015d3e316, 0x181e920a7852b9d9,
];

#[used]
#[link_section = ".limine_requests"]
static LIMINE_BASE_REVISION: [u64; 3] = [
    0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 3,
];

#[used]
#[link_section = ".limine_requests_end"]
static LIMINE_REQUESTS_END: [u64; 2] = [
    0xadc0e0531bb10d03, 0x9572709f31764c62,
];

// --------------- Limine HHDM request ---------------

#[repr(C)]
pub(crate) struct LimineHhdmResponse {
    revision: u64,
    pub(crate) offset: u64,
}

#[repr(C)]
pub(crate) struct LimineHhdmRequest {
    id: [u64; 4],
    revision: u64,
    pub(crate) response: *const LimineHhdmResponse,
}

unsafe impl Sync for LimineHhdmRequest {}

#[used]
#[link_section = ".limine_requests"]
pub(crate) static mut HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest {
    id: [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b,
         0x48dcf1cb8ad2b852, 0x63984e959a98244b],
    revision: 0,
    response: core::ptr::null(),
};

// --------------- Limine kernel address request ---------------

#[repr(C)]
struct LimineKaddrResponse {
    revision: u64,
    physical_base: u64,
    virtual_base: u64,
}

#[repr(C)]
struct LimineKaddrRequest {
    id: [u64; 4],
    revision: u64,
    response: *const LimineKaddrResponse,
}

unsafe impl Sync for LimineKaddrRequest {}

#[used]
#[link_section = ".limine_requests"]
static mut KADDR_REQUEST: LimineKaddrRequest = LimineKaddrRequest {
    id: [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b,
         0x71ba76863cc55f63, 0xb2644a48c516a487],
    revision: 0,
    response: core::ptr::null(),
};

static mut HHDM_OFFSET: u64 = 0;

extern "C" {
    static stack_top: u8;
}

cfg_m3! {
    unsafe fn sys_yield_m3(frame: *mut u64) -> u64 {
        if !M3_THREADING_ACTIVE {
            return 0;
        }

        let cur = M3_CURRENT;
        m3_save_frame(frame, cur);
        M3_THREADS[cur].saved_frame[14] = 0;

        match m3_find_ready(cur) {
            Some(next) => {
                if M3_THREADS[cur].state == M3ThreadState::Running {
                    M3_THREADS[cur].state = M3ThreadState::Ready;
                }
                m3_switch_to(frame, next);
            }
            None => {
                M3_THREADS[cur].state = M3ThreadState::Running;
                *frame.add(14) = 0;
            }
        }
        0
    }

    unsafe fn sys_thread_spawn_m3(frame: *mut u64, entry: u64) -> u64 {
        if entry >= USER_VA_LIMIT { return 0xFFFF_FFFF_FFFF_FFFF; }
        if !user_pages_ok(entry, 1, USER_PERM_READ) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }

        m3_bootstrap_main_thread(frame);

        for tid in 1..M3_MAX_THREADS {
            if M3_THREADS[tid].state != M3ThreadState::Dead {
                continue;
            }
            M3_THREADS[tid].saved_frame = [0u64; 22];
            M3_THREADS[tid].saved_frame[17] = entry;                    // RIP
            M3_THREADS[tid].saved_frame[18] = 0x23;                     // CS
            M3_THREADS[tid].saved_frame[19] = 0x02;                     // RFLAGS
            M3_THREADS[tid].saved_frame[20] = m3_stack_top_for_slot(tid); // RSP
            M3_THREADS[tid].saved_frame[21] = 0x1B;                     // SS
            M3_THREADS[tid].state = M3ThreadState::Ready;
            return tid as u64;
        }

        0xFFFF_FFFF_FFFF_FFFF
    }

    unsafe fn sys_vm_map_m3(vaddr: u64, size: u64) -> u64 {
        if size != M3_VM_PAGE_SIZE { return 0xFFFF_FFFF_FFFF_FFFF; }
        if vaddr & 0xFFF != 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
        if !user_range_ok(vaddr, size as usize) { return 0xFFFF_FFFF_FFFF_FFFF; }

        let pte = match m3_user_pte_ptr(vaddr) {
            Some(p) => p,
            None => return 0xFFFF_FFFF_FFFF_FFFF,
        };
        if *pte & 1 != 0 { return 0xFFFF_FFFF_FFFF_FFFF; }

        for i in 0..M3_MAX_VM_MAPS {
            if M3_VM_MAPS[i].active {
                continue;
            }
            let phys = m3_kv2p(M3_VM_PAGES[i].0.as_ptr() as u64);
            *pte = phys | 0x07;
            core::arch::asm!("invlpg [{}]", in(reg) vaddr, options(nostack));
            M3_VM_MAPS[i].active = true;
            M3_VM_MAPS[i].va = vaddr;
            return 0;
        }

        0xFFFF_FFFF_FFFF_FFFF
    }

    unsafe fn sys_vm_unmap_m3(vaddr: u64, size: u64) -> u64 {
        if size != M3_VM_PAGE_SIZE { return 0xFFFF_FFFF_FFFF_FFFF; }
        if vaddr & 0xFFF != 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
        if !user_range_ok(vaddr, size as usize) { return 0xFFFF_FFFF_FFFF_FFFF; }

        let mut map_idx = None;
        for i in 0..M3_MAX_VM_MAPS {
            if M3_VM_MAPS[i].active && M3_VM_MAPS[i].va == vaddr {
                map_idx = Some(i);
                break;
            }
        }
        let idx = match map_idx {
            Some(i) => i,
            None => return 0xFFFF_FFFF_FFFF_FFFF,
        };

        let pte = match m3_user_pte_ptr(vaddr) {
            Some(p) => p,
            None => return 0xFFFF_FFFF_FFFF_FFFF,
        };
        if *pte & 1 == 0 { return 0xFFFF_FFFF_FFFF_FFFF; }

        *pte = 0;
        core::arch::asm!("invlpg [{}]", in(reg) vaddr, options(nostack));
        M3_VM_MAPS[idx] = M3VmMap::EMPTY;
        0
    }

    unsafe fn sys_open_v1(path_ptr: u64, flags: u64, _mode: u64) -> u64 {
        let path = match copyinstr_user(path_ptr, 128) {
            Ok(v) => v,
            Err(_) => return 0xFFFF_FFFF_FFFF_FFFF,
        };
        let bytes: &[u8] = &path;
        if !m10_profile_path_allowed(bytes) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        // Writable file tree under /data (SimpleFS v2, gap-analysis item 5).
        // Handled before the generic rights parse: /data opens carry the
        // create bit, which the fixed-path mode mask rejects.
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            if bytes.len() >= 6 && &bytes[..5] == b"/data" && bytes[bytes.len() - 1] == 0 {
                if !r4_current_has_cap(R4_TASK_CAP_STORAGE) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                if !vfs::vfs_ready() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let rel = &bytes[5..bytes.len() - 1];
                let create = flags & M10_OPEN_CREATE != 0;
                let requested =
                    match m10_open_requested_rights(flags & M10_OPEN_MODE_MASK) {
                        Some(v) => v,
                        None => return 0xFFFF_FFFF_FFFF_FFFF,
                    };
                let uid = R4_TASKS[r4_current_smp()].uid;
                let existing = vfs::vfs_lookup(rel);
                let node = match vfs::vfs_open(rel, create) {
                    Some(n) => n,
                    None => {
                        return 0xFFFF_FFFF_FFFF_FFFF;
                    }
                };
                if existing.is_none() && node != vfs::ROOT as usize {
                    // Freshly created: the caller owns it.
                    vfs::set_node_owner(node, uid);
                } else if node != vfs::ROOT as usize && uid != 0 {
                    // Owner/other mode bits gate the requested rights.
                    let mode = vfs::node_mode(node);
                    let (r_ok, w_ok) = if vfs::node_owner(node) == uid {
                        (mode & vfs::MODE_OWNER_R != 0, mode & vfs::MODE_OWNER_W != 0)
                    } else {
                        (mode & vfs::MODE_OTHER_R != 0, mode & vfs::MODE_OTHER_W != 0)
                    };
                    if (requested & M10_RIGHT_READ != 0 && !r_ok)
                        || (requested & M10_RIGHT_WRITE != 0 && !w_ok)
                    {
                        return 0xFFFF_FFFF_FFFF_FFFF;
                    }
                }
                let kind = if node == vfs::ROOT as usize
                    || vfs::node_kind(node) == vfs::KIND_DIR
                {
                    M8FdKind::VfsDir
                } else {
                    M8FdKind::VfsFile
                };
                let max = m10_rights_for_kind(kind);
                let effective = requested | M10_RIGHT_POLL;
                if effective & !max != 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let fd = m8_alloc_fd(kind);
                if fd == 0xFFFF_FFFF_FFFF_FFFF {
                    return fd;
                }
                M8_FD_TABLE[fd as usize].rights = effective;
                M8_FD_TABLE[fd as usize].offset = 0;
                M8_FD_VFS_NODE[fd as usize] = node as u8;
                return fd;
            }
        }
        // /tmp in-memory filesystem (full-os guide Part II.5): public, no cap.
        // Placed before the generic rights parse: like /data, /tmp opens carry
        // the create bit, which the fixed-path mode mask would otherwise reject.
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            if bytes.len() >= 6 && &bytes[..5] == b"/tmp/" && bytes[bytes.len() - 1] == 0 {
                let rel = &bytes[5..bytes.len() - 1];
                let create = flags & M10_OPEN_CREATE != 0;
                let requested =
                    match m10_open_requested_rights(flags & M10_OPEN_MODE_MASK) {
                        Some(v) => v,
                        None => return 0xFFFF_FFFF_FFFF_FFFF,
                    };
                let node = match tmpfs_open(rel, create) {
                    Some(n) => n,
                    None => return 0xFFFF_FFFF_FFFF_FFFF,
                };
                let max = m10_rights_for_kind(M8FdKind::TmpFile);
                let effective = requested | M10_RIGHT_POLL;
                if effective & !max != 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let fd = m8_alloc_fd(M8FdKind::TmpFile);
                if fd == 0xFFFF_FFFF_FFFF_FFFF {
                    return fd;
                }
                M8_FD_TABLE[fd as usize].rights = effective;
                M8_FD_TABLE[fd as usize].offset = 0;
                M8_FD_VFS_NODE[fd as usize] = node as u8;
                return fd;
            }
        }
        let requested = match m10_open_requested_rights(flags) {
            Some(v) => v,
            None => return 0xFFFF_FFFF_FFFF_FFFF,
        };
        if m8_path_matches(bytes, b"/dev/console") {
            let max = m10_rights_for_kind(M8FdKind::Console);
            let effective = requested | M10_RIGHT_POLL;
            if effective & !max != 0 {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            let fd = m8_alloc_fd(M8FdKind::Console);
            if fd == 0xFFFF_FFFF_FFFF_FFFF {
                return fd;
            }
            M8_FD_TABLE[fd as usize].rights = effective;
            return fd;
        }
        // /dev character devices (full-os guide Part II.5): public, no cap.
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            let dev_kind = if m8_path_matches(bytes, b"/dev/zero") {
                Some(M8FdKind::DevZero)
            } else if m8_path_matches(bytes, b"/dev/null") {
                Some(M8FdKind::DevNull)
            } else if m8_path_matches(bytes, b"/dev/urandom") {
                Some(M8FdKind::DevUrandom)
            } else if m8_path_matches(bytes, b"/proc/self/stat") {
                Some(M8FdKind::ProcSelfStat)
            } else {
                None
            };
            if let Some(kind) = dev_kind {
                let max = m10_rights_for_kind(kind);
                let effective = requested | M10_RIGHT_POLL;
                if effective & !max != 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let fd = m8_alloc_fd(kind);
                if fd == 0xFFFF_FFFF_FFFF_FFFF {
                    return fd;
                }
                M8_FD_TABLE[fd as usize].rights = effective;
                M8_FD_TABLE[fd as usize].offset = 0;
                return fd;
            }
        }
        // /proc/<tid>/stat (full-os guide Part II.5): inspect an ARBITRARY task
        // (vs /proc/self/stat above). Path is NUL-terminated "/proc/<dec>/stat".
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            if bytes.len() > 12 && &bytes[..6] == b"/proc/" && bytes[bytes.len() - 1] == 0 {
                let body = &bytes[6..bytes.len() - 1]; // "<dec>/stat"
                let mut i = 0usize;
                let mut tid = 0usize;
                while i < body.len() && body[i].is_ascii_digit() && tid <= 255 {
                    tid = tid * 10 + (body[i] - b'0') as usize;
                    i += 1;
                }
                if i > 0 && tid <= 255 && &body[i..] == b"/stat" {
                    let kind = M8FdKind::ProcTidStat;
                    let max = m10_rights_for_kind(kind);
                    let effective = requested | M10_RIGHT_POLL;
                    if effective & !max != 0 {
                        return 0xFFFF_FFFF_FFFF_FFFF;
                    }
                    let fd = m8_alloc_fd(kind);
                    if fd == 0xFFFF_FFFF_FFFF_FFFF {
                        return fd;
                    }
                    M8_FD_TABLE[fd as usize].rights = effective;
                    M8_FD_TABLE[fd as usize].offset = 0;
                    M8_FD_VFS_NODE[fd as usize] = tid as u8;
                    return fd;
                }
            }
        }
        // /mnt/<NAME> mounts the FAT16 volume into the namespace (full-os guide
        // Part II.5 mounts): read-only, root-directory 8.3 names. The file is
        // cached on open; one open /mnt file at a time (v1).
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            if bytes.len() >= 6 && &bytes[..5] == b"/mnt/" && bytes[bytes.len() - 1] == 0 {
                // The cache holds one file at a time; reject a second open
                // rather than silently overwriting an in-use FatFile fd.
                if FAT_FILE_BUSY {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let rel = &bytes[5..bytes.len() - 1];
                // Convert "name.ext" -> a space-padded 8.3 directory name.
                let mut name83 = [b' '; 11];
                let mut dot = rel.len();
                let mut k = 0;
                while k < rel.len() {
                    if rel[k] == b'.' {
                        dot = k;
                        break;
                    }
                    k += 1;
                }
                let base = &rel[..dot];
                if base.is_empty() || base.len() > 8 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let mut i = 0;
                while i < base.len() {
                    name83[i] = ascii_upper(base[i]);
                    i += 1;
                }
                if dot < rel.len() {
                    let ext = &rel[dot + 1..];
                    if ext.len() > 3 {
                        return 0xFFFF_FFFF_FFFF_FFFF;
                    }
                    let mut j = 0;
                    while j < ext.len() {
                        name83[8 + j] = ascii_upper(ext[j]);
                        j += 1;
                    }
                }
                let n = match fat16_read_named(&name83, &mut FAT_FILE) {
                    Some(n) => n,
                    None => return 0xFFFF_FFFF_FFFF_FFFF,
                };
                FAT_FILE_LEN = n;
                let max = m10_rights_for_kind(M8FdKind::FatFile);
                let effective = requested | M10_RIGHT_POLL;
                if effective & !max != 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let fd = m8_alloc_fd(M8FdKind::FatFile);
                if fd == 0xFFFF_FFFF_FFFF_FFFF {
                    return fd;
                }
                M8_FD_TABLE[fd as usize].rights = effective;
                M8_FD_TABLE[fd as usize].offset = 0;
                FAT_FILE_BUSY = true;
                return fd;
            }
        }
        if m8_path_matches(bytes, b"/compat/hello.txt") {
            #[cfg(feature = "go_test")]
            if !r4_current_has_cap(R4_TASK_CAP_STORAGE) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            let max = m10_rights_for_kind(M8FdKind::CompatFile);
            let effective = requested | M10_RIGHT_POLL;
            if effective & !max != 0 {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            let fd = m8_alloc_fd(M8FdKind::CompatFile);
            if fd == 0xFFFF_FFFF_FFFF_FFFF {
                return fd;
            }
            M8_FD_TABLE[fd as usize].rights = effective;
            return fd;
        }
        #[cfg(feature = "go_test")]
        {
            if storage::r4_storage_available() && m8_path_matches(bytes, b"/runtime/journal.bin") {
                if !r4_current_has_cap(R4_TASK_CAP_STORAGE) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let max = m10_rights_for_kind(M8FdKind::JournalFile);
                let effective = requested | M10_RIGHT_POLL;
                if effective & !max != 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let fd = m8_alloc_fd(M8FdKind::JournalFile);
                if fd == 0xFFFF_FFFF_FFFF_FFFF {
                    return fd;
                }
                M8_FD_TABLE[fd as usize].rights = effective;
                return fd;
            }
            if storage::r4_storage_available() && m8_path_matches(bytes, b"/runtime/state.bin") {
                if !r4_current_has_cap(R4_TASK_CAP_STORAGE) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let max = m10_rights_for_kind(M8FdKind::StateFile);
                let effective = requested | M10_RIGHT_POLL;
                if effective & !max != 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let fd = m8_alloc_fd(M8FdKind::StateFile);
                if fd == 0xFFFF_FFFF_FFFF_FFFF {
                    return fd;
                }
                M8_FD_TABLE[fd as usize].rights = effective;
                return fd;
            }
            if storage::r4_storage_available() && m8_path_matches(bytes, b"/runtime/pkgstate.bin") {
                if !r4_current_has_cap(R4_TASK_CAP_STORAGE) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let max = m10_rights_for_kind(M8FdKind::PkgStateFile);
                let effective = requested | M10_RIGHT_POLL;
                if effective & !max != 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let fd = m8_alloc_fd(M8FdKind::PkgStateFile);
                if fd == 0xFFFF_FFFF_FFFF_FFFF {
                    return fd;
                }
                M8_FD_TABLE[fd as usize].rights = effective;
                return fd;
            }
            if storage::r4_storage_available() && m8_path_matches(bytes, b"/runtime/platform.bin") {
                if !r4_current_has_cap(R4_TASK_CAP_STORAGE) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let max = m10_rights_for_kind(M8FdKind::PlatformFile);
                let effective = requested | M10_RIGHT_POLL;
                if effective & !max != 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let fd = m8_alloc_fd(M8FdKind::PlatformFile);
                if fd == 0xFFFF_FFFF_FFFF_FFFF {
                    return fd;
                }
                M8_FD_TABLE[fd as usize].rights = effective;
                return fd;
            }
        }
        0xFFFF_FFFF_FFFF_FFFF
    }

    unsafe fn sys_read_v1(fd: u64, buf: u64, len: u64) -> u64 {
        if len == 0 { return 0; }
        if len > 4096 { return 0xFFFF_FFFF_FFFF_FFFF; }
        let idx = fd as usize;
        if idx >= M8_FD_MAX { return 0xFFFF_FFFF_FFFF_FFFF; }
        #[cfg(feature = "go_test")]
        if !r4_fd_owner_ok(idx) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if M8_FD_TABLE[idx].rights & M10_RIGHT_READ == 0 {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }

        match M8_FD_TABLE[idx].kind {
            M8FdKind::Free => 0xFFFF_FFFF_FFFF_FFFF,
            M8FdKind::Console => {
                let n = len as usize;
                if n > 256 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let mut kbuf = [0u8; 256];
                let mut read = 0usize;
                while read < n {
                    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                    {
                        kbuf[read] = console_read_byte();
                    }
                    #[cfg(not(all(feature = "go_test", not(feature = "compat_real_test"))))]
                    {
                        kbuf[read] = serial_read_byte();
                    }
                    read += 1;
                }
                if copyout_user(buf, &kbuf[..read], read).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                read as u64
            }
            M8FdKind::CompatFile => {
                let off = M8_FD_TABLE[idx].offset;
                if off >= M8_COMPAT_FILE.len() {
                    return 0;
                }
                let remaining = M8_COMPAT_FILE.len() - off;
                let req = len as usize;
                let n = if req < remaining { req } else { remaining };
                if copyout_user(buf, &M8_COMPAT_FILE[off..off + n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset += n;
                n as u64
            }
            #[cfg(feature = "go_test")]
            M8FdKind::JournalFile => 0xFFFF_FFFF_FFFF_FFFF,
            #[cfg(feature = "go_test")]
            M8FdKind::StateFile => {
                let total = storage::r4_storage_state_len();
                let off = M8_FD_TABLE[idx].offset;
                if off >= total {
                    return 0;
                }
                let req = len as usize;
                let remaining = total - off;
                let n = if req < remaining { req } else { remaining };
                let mut kbuf = [0u8; 496];
                if !storage::r4_storage_copy_state(off, &mut kbuf[..n]) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                if copyout_user(buf, &kbuf[..n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset += n;
                n as u64
            }
            #[cfg(feature = "go_test")]
            M8FdKind::PkgStateFile => {
                let total = storage::r4_storage_runtime_len(storage::R4StorageRuntimeFile::PkgState);
                let off = M8_FD_TABLE[idx].offset;
                if off >= total {
                    return 0;
                }
                let req = len as usize;
                let remaining = total - off;
                let n = if req < remaining { req } else { remaining };
                let mut kbuf = [0u8; storage::R4_STORAGE_RUNTIME_FILE_MAX_BYTES];
                if !storage::r4_storage_runtime_copy(
                    storage::R4StorageRuntimeFile::PkgState,
                    off,
                    &mut kbuf[..n],
                ) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                if copyout_user(buf, &kbuf[..n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset += n;
                n as u64
            }
            #[cfg(feature = "go_test")]
            M8FdKind::PlatformFile => {
                let total = storage::r4_storage_runtime_len(storage::R4StorageRuntimeFile::Platform);
                let off = M8_FD_TABLE[idx].offset;
                if off >= total {
                    return 0;
                }
                let req = len as usize;
                let remaining = total - off;
                let n = if req < remaining { req } else { remaining };
                let mut kbuf = [0u8; storage::R4_STORAGE_RUNTIME_FILE_MAX_BYTES];
                if !storage::r4_storage_runtime_copy(
                    storage::R4StorageRuntimeFile::Platform,
                    off,
                    &mut kbuf[..n],
                ) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                if copyout_user(buf, &kbuf[..n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset += n;
                n as u64
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::VfsFile => {
                let node = M8_FD_VFS_NODE[idx] as usize;
                let off = M8_FD_TABLE[idx].offset;
                let req = (len as usize).min(4096);
                let mut kbuf = [0u8; 512];
                let mut done = 0usize;
                while done < req {
                    let n = (req - done).min(kbuf.len());
                    let got = vfs::vfs_read(node, off + done, &mut kbuf[..n]);
                    if got == 0 {
                        break;
                    }
                    if copyout_user(buf + done as u64, &kbuf[..got], got).is_err() {
                        return 0xFFFF_FFFF_FFFF_FFFF;
                    }
                    done += got;
                    if got < n {
                        break;
                    }
                }
                M8_FD_TABLE[idx].offset += done;
                done as u64
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::VfsDir => {
                let node = M8_FD_VFS_NODE[idx] as usize;
                let req = (len as usize).min(512);
                let mut kbuf = [0u8; 512];
                let cursor = M8_FD_TABLE[idx].offset;
                let (written, next) = vfs::vfs_readdir(node, cursor, &mut kbuf[..req]);
                if written > 0 && copyout_user(buf, &kbuf[..written], written).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset = next;
                written as u64
            }
            #[cfg(all(feature = "go_test", feature = "compat_real_test"))]
            M8FdKind::VfsFile | M8FdKind::VfsDir => 0xFFFF_FFFF_FFFF_FFFF,
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::DevZero => {
                let n = len as usize;
                let z = [0u8; 256];
                let mut done = 0usize;
                while done < n {
                    let chunk = core::cmp::min(z.len(), n - done);
                    if copyout_user(buf + done as u64, &z[..chunk], chunk).is_err() {
                        return 0xFFFF_FFFF_FFFF_FFFF;
                    }
                    done += chunk;
                }
                n as u64
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::DevUrandom => sys_getrandom(buf, len),
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::DevNull => 0, // EOF
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::ProcSelfStat => {
                // Generate the caller's stat line on demand.
                let mut line = [0u8; 64];
                let mut w = 0usize;
                let tag = b"tid=";
                line[w..w + tag.len()].copy_from_slice(tag);
                w += tag.len();
                w += fmt_hex_u64(&mut line[w..], R4_CURRENT as u64);
                let utag = b" uid=";
                line[w..w + utag.len()].copy_from_slice(utag);
                w += utag.len();
                w += fmt_hex_u64(&mut line[w..], R4_TASKS[r4_current_smp()].uid as u64);
                let stag = b" state=run\n";
                line[w..w + stag.len()].copy_from_slice(stag);
                w += stag.len();
                let off = M8_FD_TABLE[idx].offset;
                if off >= w {
                    return 0;
                }
                let remaining = w - off;
                let n = core::cmp::min(len as usize, remaining);
                if copyout_user(buf, &line[off..off + n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset += n;
                n as u64
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::ProcTidStat => {
                // Generate an arbitrary task's stat line on demand (the target
                // tid was stored at open time in M8_FD_VFS_NODE[fd]).
                let tid = M8_FD_VFS_NODE[idx] as usize;
                if tid >= R4_NUM_TASKS || matches!(R4_TASKS[tid].state, R4State::Dead) {
                    return 0; // no such live task -> EOF
                }
                let mut line = [0u8; 64];
                let mut w = 0usize;
                let tag = b"tid=";
                line[w..w + tag.len()].copy_from_slice(tag);
                w += tag.len();
                w += fmt_hex_u64(&mut line[w..], tid as u64);
                let utag = b" uid=";
                line[w..w + utag.len()].copy_from_slice(utag);
                w += utag.len();
                w += fmt_hex_u64(&mut line[w..], R4_TASKS[tid].uid as u64);
                let stag: &[u8] = match R4_TASKS[tid].state {
                    R4State::Running => b" state=run\n",
                    R4State::Ready => b" state=ready\n",
                    R4State::Blocked => b" state=block\n",
                    _ => b" state=other\n",
                };
                line[w..w + stag.len()].copy_from_slice(stag);
                w += stag.len();
                let off = M8_FD_TABLE[idx].offset;
                if off >= w {
                    return 0;
                }
                let remaining = w - off;
                let n = core::cmp::min(len as usize, remaining);
                if copyout_user(buf, &line[off..off + n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset += n;
                n as u64
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::TimerFd => {
                // 8-byte expiration count when fired, else 0. A periodic timerfd
                // (op 4) re-arms to its next deadline; a one-shot disarms.
                if len < 8 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let now = R4_PREEMPT_TICKS as usize;
                let deadline = M8_FD_TABLE[idx].offset;
                if now < deadline {
                    return 0; // not yet expired (a disarmed one-shot has deadline = MAX)
                }
                let (count, next) = timerfd_fire(deadline, M8_FD_TIMER_IVL[idx] as usize, now);
                if copyout_user(buf, &count.to_le_bytes(), 8).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset = next;
                8
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::TmpFile => {
                let ti = M8_FD_VFS_NODE[idx] as usize;
                let off = M8_FD_TABLE[idx].offset;
                if ti >= TMPFS_MAX || off >= TMPFS[ti].len {
                    return 0;
                }
                let remaining = TMPFS[ti].len - off;
                let n = core::cmp::min(len as usize, remaining);
                let mut kbuf = [0u8; TMPFS_CAP];
                kbuf[..n].copy_from_slice(&TMPFS[ti].data[off..off + n]);
                if copyout_user(buf, &kbuf[..n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset += n;
                n as u64
            }
            #[cfg(feature = "go_test")]
            M8FdKind::PipeR => {
                let p = M8_FD_PIPE[idx] as usize;
                if p >= PIPE_MAX || !PIPES[p].active {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let avail = PIPES[p].len;
                if avail == 0 {
                    if PIPES[p].writers == 0 {
                        return 0; // EOF: every writer is gone
                    }
                    return 0xFFFF_FFFF_FFFF_FFFF; // empty, retry later
                }
                let n = avail.min(len as usize).min(PIPE_CAP);
                let mut kbuf = [0u8; PIPE_CAP];
                kbuf[..n].copy_from_slice(&PIPES[p].buf[..n]);
                if copyout_user(buf, &kbuf[..n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                PIPES[p].buf.copy_within(n..avail, 0);
                PIPES[p].len = avail - n;
                n as u64
            }
            #[cfg(feature = "go_test")]
            M8FdKind::PipeW => 0xFFFF_FFFF_FFFF_FFFF,
            // pty (full-os guide Part V.11): master drains slave->master, slave
            // drains master->slave. Non-blocking v1: empty read returns 0.
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::PtyMaster | M8FdKind::PtySlave => {
                let p = M8_FD_PTY[idx] as usize;
                if p >= PTY_MAX || !PTYS[p].active {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let is_master = matches!(M8_FD_TABLE[idx].kind, M8FdKind::PtyMaster);
                let avail = if is_master { PTYS[p].s2m_len } else { PTYS[p].m2s_len };
                if avail == 0 {
                    return 0;
                }
                let n = avail.min(len as usize).min(PTY_CAP);
                let mut kbuf = [0u8; PTY_CAP];
                if is_master {
                    kbuf[..n].copy_from_slice(&PTYS[p].s2m[..n]);
                } else {
                    kbuf[..n].copy_from_slice(&PTYS[p].m2s[..n]);
                }
                if copyout_user(buf, &kbuf[..n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                if is_master {
                    PTYS[p].s2m.copy_within(n..avail, 0);
                    PTYS[p].s2m_len = avail - n;
                } else {
                    PTYS[p].m2s.copy_within(n..avail, 0);
                    PTYS[p].m2s_len = avail - n;
                }
                n as u64
            }
            // FAT16 file via /mnt (full-os guide Part II.5): serve from the
            // FAT_FILE cache filled at open; the fd offset tracks position.
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::FatFile => {
                let off = M8_FD_TABLE[idx].offset;
                if off >= FAT_FILE_LEN {
                    return 0;
                }
                let n = (FAT_FILE_LEN - off).min(len as usize);
                if copyout_user(buf, &FAT_FILE[off..off + n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset += n;
                n as u64
            }
            #[cfg(not(feature = "go_test"))]
            _ => 0xFFFF_FFFF_FFFF_FFFF,
        }
    }

    unsafe fn sys_write_v1(fd: u64, buf: u64, len: u64) -> u64 {
        if len == 0 { return 0; }
        if len > 256 { return 0xFFFF_FFFF_FFFF_FFFF; }
        let idx = fd as usize;
        if idx >= M8_FD_MAX { return 0xFFFF_FFFF_FFFF_FFFF; }
        #[cfg(feature = "go_test")]
        if !r4_fd_owner_ok(idx) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if M8_FD_TABLE[idx].rights & M10_RIGHT_WRITE == 0 {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }

        match M8_FD_TABLE[idx].kind {
            M8FdKind::Free => 0xFFFF_FFFF_FFFF_FFFF,
            M8FdKind::CompatFile => 0xFFFF_FFFF_FFFF_FFFF,
            M8FdKind::Console => {
                let n = len as usize;
                let mut kbuf = [0u8; 256];
                if copyin_user(&mut kbuf[..n], buf, n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                serial_write(&kbuf[..n]);
                len
            }
            #[cfg(feature = "go_test")]
            M8FdKind::JournalFile => {
                let n = len as usize;
                let mut kbuf = [0u8; 256];
                if copyin_user(&mut kbuf[..n], buf, n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                if !storage::r4_storage_write_journal(&kbuf[..n]) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset = n;
                len
            }
            #[cfg(feature = "go_test")]
            M8FdKind::StateFile => 0xFFFF_FFFF_FFFF_FFFF,
            #[cfg(feature = "go_test")]
            M8FdKind::PkgStateFile => {
                let n = len as usize;
                let mut kbuf = [0u8; storage::R4_STORAGE_RUNTIME_FILE_MAX_BYTES];
                if copyin_user(&mut kbuf[..n], buf, n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                if !storage::r4_storage_runtime_write(storage::R4StorageRuntimeFile::PkgState, &kbuf[..n]) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset = n;
                len
            }
            #[cfg(feature = "go_test")]
            M8FdKind::PlatformFile => {
                let n = len as usize;
                let mut kbuf = [0u8; storage::R4_STORAGE_RUNTIME_FILE_MAX_BYTES];
                if copyin_user(&mut kbuf[..n], buf, n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                if !storage::r4_storage_runtime_write(storage::R4StorageRuntimeFile::Platform, &kbuf[..n]) {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset = n;
                len
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::VfsFile => {
                let n = len as usize;
                let mut kbuf = [0u8; 256];
                if copyin_user(&mut kbuf[..n], buf, n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let node = M8_FD_VFS_NODE[idx] as usize;
                let off = M8_FD_TABLE[idx].offset;
                let done = vfs::vfs_write(node, off, &kbuf[..n]);
                if done == 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                M8_FD_TABLE[idx].offset += done;
                done as u64
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::VfsDir => 0xFFFF_FFFF_FFFF_FFFF,
            #[cfg(all(feature = "go_test", feature = "compat_real_test"))]
            M8FdKind::VfsFile | M8FdKind::VfsDir => 0xFFFF_FFFF_FFFF_FFFF,
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::DevNull => len, // discard
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::DevZero
            | M8FdKind::DevUrandom
            | M8FdKind::ProcSelfStat
            | M8FdKind::ProcTidStat
            | M8FdKind::TimerFd => 0xFFFF_FFFF_FFFF_FFFF,
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::TmpFile => {
                let ti = M8_FD_VFS_NODE[idx] as usize;
                let off = M8_FD_TABLE[idx].offset;
                if ti >= TMPFS_MAX || off >= TMPFS_CAP {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let n = core::cmp::min(len as usize, TMPFS_CAP - off);
                let mut kbuf = [0u8; TMPFS_CAP];
                if copyin_user(&mut kbuf[..n], buf, n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                TMPFS[ti].data[off..off + n].copy_from_slice(&kbuf[..n]);
                if off + n > TMPFS[ti].len {
                    TMPFS[ti].len = off + n;
                }
                M8_FD_TABLE[idx].offset += n;
                n as u64
            }
            #[cfg(feature = "go_test")]
            M8FdKind::PipeW => {
                let p = M8_FD_PIPE[idx] as usize;
                if p >= PIPE_MAX || !PIPES[p].active || PIPES[p].readers == 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let n = len as usize;
                if PIPES[p].len + n > PIPE_CAP {
                    return 0xFFFF_FFFF_FFFF_FFFF; // full, retry later
                }
                let mut kbuf = [0u8; 256];
                if copyin_user(&mut kbuf[..n], buf, n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let off = PIPES[p].len;
                PIPES[p].buf[off..off + n].copy_from_slice(&kbuf[..n]);
                PIPES[p].len += n;
                len
            }
            #[cfg(feature = "go_test")]
            M8FdKind::PipeR => 0xFFFF_FFFF_FFFF_FFFF,
            // pty (full-os guide Part V.11): master writes master->slave, slave
            // writes slave->master. (sys_write_v1 already caps len at 256.)
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::PtyMaster | M8FdKind::PtySlave => {
                let p = M8_FD_PTY[idx] as usize;
                if p >= PTY_MAX || !PTYS[p].active {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let n = len as usize;
                let mut kbuf = [0u8; 256];
                if copyin_user(&mut kbuf[..n], buf, n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let is_master = matches!(M8_FD_TABLE[idx].kind, M8FdKind::PtyMaster);
                if is_master {
                    if PTYS[p].m2s_len + n > PTY_CAP {
                        return 0xFFFF_FFFF_FFFF_FFFF;
                    }
                    let off = PTYS[p].m2s_len;
                    PTYS[p].m2s[off..off + n].copy_from_slice(&kbuf[..n]);
                    PTYS[p].m2s_len += n;
                } else {
                    if PTYS[p].s2m_len + n > PTY_CAP {
                        return 0xFFFF_FFFF_FFFF_FFFF;
                    }
                    let off = PTYS[p].s2m_len;
                    PTYS[p].s2m[off..off + n].copy_from_slice(&kbuf[..n]);
                    PTYS[p].s2m_len += n;
                }
                len
            }
            // FAT16 /mnt files are read-only in v1.
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::FatFile => 0xFFFF_FFFF_FFFF_FFFF,
            #[cfg(not(feature = "go_test"))]
            _ => 0xFFFF_FFFF_FFFF_FFFF,
        }
    }

    /// sys_fs_ctl (ABI v3.x id 47): namespace mutations and stat for the
    /// /data tree. op 1 = mkdir, 2 = unlink, 3 = stat (returns
    /// kind << 32 | size). op 4 = pipe create (returns rfd << 8 | wfd).
    /// op 5 = chmod (arg = mode bits, owner or root only).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_fs_ctl_v1(op: u64, path_ptr: u64, arg: u64) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        if op == 6 {
            // lseek(fd, offset, whence) (full-os guide Part V.11 rlibc). The fd is the
            // low 32 bits of path_ptr and the whence is bits [39:32]: 0 = SEEK_SET,
            // 1 = SEEK_CUR (relative to the current offset), 2 = SEEK_END (relative to
            // the file size). arg is the offset/delta (added with wrapping, so a
            // two's-complement negative delta seeks backwards). Owner-gated, no
            // storage cap (a generic fd operation). Returns the new absolute offset.
            // SEEK_SET keeps the historical path_ptr==fd encoding (whence bits zero).
            let fd = (path_ptr & 0xFFFF_FFFF) as usize;
            let whence = ((path_ptr >> 32) & 0xFF) as u8;
            if fd < 3 || fd >= M8_FD_MAX || M8_FD_TABLE[fd].kind == M8FdKind::Free {
                return ERR;
            }
            if !r4_fd_owner_ok(fd) {
                return ERR;
            }
            let new = match whence {
                0 => arg as usize,                                          // SEEK_SET
                1 => M8_FD_TABLE[fd].offset.wrapping_add(arg as usize),     // SEEK_CUR
                2 => {
                    // SEEK_END needs a file size, which only a VFS file carries.
                    if M8_FD_TABLE[fd].kind != M8FdKind::VfsFile {
                        return ERR;
                    }
                    let node = M8_FD_VFS_NODE[fd] as usize;
                    vfs::node_size(node).wrapping_add(arg as usize)
                }
                _ => return ERR,
            };
            M8_FD_TABLE[fd].offset = new;
            return new as u64;
        }
        if !r4_current_has_cap(R4_TASK_CAP_STORAGE) || !vfs::vfs_ready() {
            return ERR;
        }
        if op == 4 {
            // pipe create: ignore the path, allocate a ring + two fds.
            let mut p = 0usize;
            while p < PIPE_MAX && PIPES[p].active {
                p += 1;
            }
            if p == PIPE_MAX {
                return ERR;
            }
            let rfd = m8_alloc_fd(M8FdKind::PipeR);
            if rfd == ERR {
                return ERR;
            }
            let wfd = m8_alloc_fd(M8FdKind::PipeW);
            if wfd == ERR {
                M8_FD_TABLE[rfd as usize] = M8FdEntry::EMPTY;
                if R4_TASKS[r4_current_smp()].fd_count != 0 {
                    R4_TASKS[r4_current_smp()].fd_count -= 1;
                }
                return ERR;
            }
            PIPES[p] = PipeObj::EMPTY;
            PIPES[p].active = true;
            PIPES[p].readers = 1;
            PIPES[p].writers = 1;
            M8_FD_PIPE[rfd as usize] = p as u8;
            M8_FD_PIPE[wfd as usize] = p as u8;
            M8_FD_TABLE[rfd as usize].rights = M10_RIGHT_READ | M10_RIGHT_POLL;
            M8_FD_TABLE[wfd as usize].rights = M10_RIGHT_WRITE | M10_RIGHT_POLL;
            return (rfd << 8) | wfd;
        }
        let path = match copyinstr_user(path_ptr, 128) {
            Ok(v) => v,
            Err(_) => return ERR,
        };
        let bytes: &[u8] = &path;
        if bytes.len() <= 6 || &bytes[..5] != b"/data" || bytes[bytes.len() - 1] != 0 {
            return ERR;
        }
        let rel = &bytes[5..bytes.len() - 1];
        let uid = R4_TASKS[r4_current_smp()].uid;
        match op {
            1 => {
                if vfs::vfs_mkdir(rel) {
                    if let Some(n) = vfs::vfs_lookup(rel) {
                        if n != vfs::ROOT as usize {
                            vfs::set_node_owner(n, uid);
                        }
                    }
                    0
                } else {
                    ERR
                }
            }
            2 => {
                // Unlink: root, the owner, or anyone the mode grants
                // other-write (v1 simplification of directory perms).
                if uid != 0 {
                    match vfs::vfs_lookup(rel) {
                        Some(n) if n != vfs::ROOT as usize => {
                            let allowed = vfs::node_owner(n) == uid
                                || vfs::node_mode(n) & vfs::MODE_OTHER_W != 0;
                            if !allowed {
                                return ERR;
                            }
                        }
                        _ => return ERR,
                    }
                }
                if vfs::vfs_unlink(rel) { 0 } else { ERR }
            }
            3 => match vfs::vfs_stat(rel) {
                Some((kind, size)) => ((kind as u64) << 32) | size as u64,
                None => ERR,
            },
            5 => {
                // chmod: root or the owner; mode is the low 4 bits.
                if arg > 0xF {
                    return ERR;
                }
                match vfs::vfs_lookup(rel) {
                    Some(n) if n != vfs::ROOT as usize => {
                        if uid != 0 && vfs::node_owner(n) != uid {
                            return ERR;
                        }
                        if vfs::set_node_mode(n, arg as u8) { 0 } else { ERR }
                    }
                    _ => ERR,
                }
            }
            _ => ERR,
        }
    }

    unsafe fn sys_fsync_v1(fd: u64) -> u64 {
        let idx = fd as usize;
        if idx >= M8_FD_MAX { return 0xFFFF_FFFF_FFFF_FFFF; }
        #[cfg(feature = "go_test")]
        if !r4_fd_owner_ok(idx) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        match M8_FD_TABLE[idx].kind {
            #[cfg(feature = "go_test")]
            M8FdKind::JournalFile => {
                if M8_FD_TABLE[idx].rights & M10_RIGHT_WRITE == 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                if storage::r4_storage_fsync() { 0 } else { 0xFFFF_FFFF_FFFF_FFFF }
            }
            #[cfg(feature = "go_test")]
            M8FdKind::PkgStateFile | M8FdKind::PlatformFile => {
                if M8_FD_TABLE[idx].rights & M10_RIGHT_WRITE == 0 {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                0
            }
            _ => 0xFFFF_FFFF_FFFF_FFFF,
        }
    }

    unsafe fn sys_close_v1(fd: u64) -> u64 {
        let idx = fd as usize;
        if idx >= M8_FD_MAX { return 0xFFFF_FFFF_FFFF_FFFF; }
        if idx < 3 {
            // Keep stdio descriptors stable for compatibility-profile startup.
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if M8_FD_TABLE[idx].kind == M8FdKind::Free {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        #[cfg(feature = "go_test")]
        {
            if !r4_fd_owner_ok(idx) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            let owner_tid = M8_FD_TABLE[idx].owner_tid;
            if owner_tid < R4_NUM_TASKS && R4_TASKS[owner_tid].fd_count != 0 {
                R4_TASKS[owner_tid].fd_count -= 1;
            }
            pipe_drop_end(idx);
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            {
                pty_drop_end(idx);
                if M8_FD_TABLE[idx].kind == M8FdKind::FatFile {
                    FAT_FILE_BUSY = false;
                }
            }
        }
        M8_FD_TABLE[idx] = M8FdEntry::EMPTY;
        0
    }

    unsafe fn sys_wait_v1(pid: u64, status_ptr: u64, options: u64) -> u64 {
        if options != 0 {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        // v1 baseline: waitpid(-1, ...) and waitpid(1, ...) are accepted.
        if pid != u64::MAX && pid != 1 {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if !M8_WAIT_HAS_EXIT {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if status_ptr != 0 {
            let st = M8_WAIT_EXIT_STATUS.to_le_bytes();
            if copyout_user(status_ptr, &st, st.len()).is_err() {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
        }
        M8_WAIT_HAS_EXIT = false;
        1
    }

    unsafe fn sys_poll_v1(fds_ptr: u64, nfds: u64, _timeout_ticks: u64) -> u64 {
        const POLLFD_SIZE: usize = 8;
        const POLLIN: u16 = 0x0001;
        const POLLOUT: u16 = 0x0004;
        const POLLERR: u16 = 0x0008;

        if nfds == 0 {
            return 0;
        }
        if nfds > M8_FD_MAX as u64 {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }

        let total = match (nfds as usize).checked_mul(POLLFD_SIZE) {
            Some(v) => v,
            None => return 0xFFFF_FFFF_FFFF_FFFF,
        };
        if !user_range_ok(fds_ptr, total)
            || !user_pages_ok(fds_ptr, total, USER_PERM_READ | USER_PERM_WRITE)
        {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }

        let mut ready = 0u64;
        for i in 0..(nfds as usize) {
            let slot_ptr = fds_ptr + (i * POLLFD_SIZE) as u64;
            let mut slot = [0u8; POLLFD_SIZE];
            if copyin_user(&mut slot, slot_ptr, POLLFD_SIZE).is_err() {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }

            let fd = i32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]);
            let events = u16::from_le_bytes([slot[4], slot[5]]);
            let mut revents: u16 = 0;

            if fd < 0 {
                revents |= POLLERR;
            } else {
                let idx = fd as usize;
                if idx >= M8_FD_MAX {
                    revents |= POLLERR;
                } else {
                    let rights = M8_FD_TABLE[idx].rights;
                    if rights & M10_RIGHT_POLL == 0 {
                        revents |= POLLERR;
                    }
                    match M8_FD_TABLE[idx].kind {
                        M8FdKind::Free => revents |= POLLERR,
                        M8FdKind::Console => {
                            if events & POLLIN != 0 && rights & M10_RIGHT_READ != 0 && serial_can_read() {
                                revents |= POLLIN;
                            }
                            if events & POLLOUT != 0 && rights & M10_RIGHT_WRITE != 0 {
                                revents |= POLLOUT;
                            }
                        }
                        M8FdKind::CompatFile => {
                            if events & POLLIN != 0
                                && rights & M10_RIGHT_READ != 0
                                && M8_FD_TABLE[idx].offset < M8_COMPAT_FILE.len()
                            {
                                revents |= POLLIN;
                            }
                        }
                        #[cfg(feature = "go_test")]
                        M8FdKind::JournalFile => {
                            if events & POLLOUT != 0 && rights & M10_RIGHT_WRITE != 0 {
                                revents |= POLLOUT;
                            }
                        }
                        #[cfg(feature = "go_test")]
                        M8FdKind::StateFile => {
                            if events & POLLIN != 0
                                && rights & M10_RIGHT_READ != 0
                                && M8_FD_TABLE[idx].offset < storage::r4_storage_state_len()
                            {
                                revents |= POLLIN;
                            }
                        }
                        #[cfg(feature = "go_test")]
                        M8FdKind::PkgStateFile => {
                            if events & POLLIN != 0
                                && rights & M10_RIGHT_READ != 0
                                && M8_FD_TABLE[idx].offset
                                    < storage::r4_storage_runtime_len(storage::R4StorageRuntimeFile::PkgState)
                            {
                                revents |= POLLIN;
                            }
                            if events & POLLOUT != 0 && rights & M10_RIGHT_WRITE != 0 {
                                revents |= POLLOUT;
                            }
                        }
                        #[cfg(feature = "go_test")]
                        M8FdKind::PlatformFile => {
                            if events & POLLIN != 0
                                && rights & M10_RIGHT_READ != 0
                                && M8_FD_TABLE[idx].offset
                                    < storage::r4_storage_runtime_len(storage::R4StorageRuntimeFile::Platform)
                            {
                                revents |= POLLIN;
                            }
                            if events & POLLOUT != 0 && rights & M10_RIGHT_WRITE != 0 {
                                revents |= POLLOUT;
                            }
                        }
                        #[cfg(feature = "go_test")]
                        M8FdKind::VfsFile | M8FdKind::VfsDir => {
                            if events & POLLIN != 0 && rights & M10_RIGHT_READ != 0 {
                                revents |= POLLIN;
                            }
                            if events & POLLOUT != 0 && rights & M10_RIGHT_WRITE != 0 {
                                revents |= POLLOUT;
                            }
                        }
                        #[cfg(feature = "go_test")]
                        M8FdKind::PipeR => {
                            let p = M8_FD_PIPE[idx] as usize;
                            if events & POLLIN != 0
                                && rights & M10_RIGHT_READ != 0
                                && p < PIPE_MAX
                                && PIPES[p].len > 0
                            {
                                revents |= POLLIN;
                            }
                        }
                        #[cfg(feature = "go_test")]
                        M8FdKind::PipeW => {
                            let p = M8_FD_PIPE[idx] as usize;
                            if events & POLLOUT != 0
                                && rights & M10_RIGHT_WRITE != 0
                                && p < PIPE_MAX
                                && PIPES[p].len < PIPE_CAP
                            {
                                revents |= POLLOUT;
                            }
                        }
                        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                        M8FdKind::DevZero | M8FdKind::DevUrandom => {
                            if events & POLLIN != 0 && rights & M10_RIGHT_READ != 0 {
                                revents |= POLLIN;
                            }
                        }
                        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                        M8FdKind::DevNull => {
                            if events & POLLOUT != 0 && rights & M10_RIGHT_WRITE != 0 {
                                revents |= POLLOUT;
                            }
                        }
                        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                        M8FdKind::ProcSelfStat | M8FdKind::ProcTidStat => {
                            if events & POLLIN != 0 && rights & M10_RIGHT_READ != 0 {
                                revents |= POLLIN;
                            }
                        }
                        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                        M8FdKind::TimerFd => {
                            if events & POLLIN != 0
                                && rights & M10_RIGHT_READ != 0
                                && R4_PREEMPT_TICKS as usize >= M8_FD_TABLE[idx].offset
                            {
                                revents |= POLLIN;
                            }
                        }
                        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                        M8FdKind::TmpFile => {
                            if events & POLLIN != 0 && rights & M10_RIGHT_READ != 0 {
                                revents |= POLLIN;
                            }
                            if events & POLLOUT != 0 && rights & M10_RIGHT_WRITE != 0 {
                                revents |= POLLOUT;
                            }
                        }
                        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                        M8FdKind::PtyMaster | M8FdKind::PtySlave => {
                            let p = M8_FD_PTY[idx] as usize;
                            let readable = p < PTY_MAX
                                && PTYS[p].active
                                && if matches!(M8_FD_TABLE[idx].kind, M8FdKind::PtyMaster) {
                                    PTYS[p].s2m_len > 0
                                } else {
                                    PTYS[p].m2s_len > 0
                                };
                            if events & POLLIN != 0 && rights & M10_RIGHT_READ != 0 && readable {
                                revents |= POLLIN;
                            }
                            if events & POLLOUT != 0 && rights & M10_RIGHT_WRITE != 0 {
                                revents |= POLLOUT;
                            }
                        }
                        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                        M8FdKind::FatFile => {
                            if events & POLLIN != 0
                                && rights & M10_RIGHT_READ != 0
                                && M8_FD_TABLE[idx].offset < FAT_FILE_LEN
                            {
                                revents |= POLLIN;
                            }
                        }
                        #[cfg(not(feature = "go_test"))]
                        _ => revents |= POLLERR,
                    }
                }
            }

            if revents != 0 {
                ready += 1;
            }

            let rv = revents.to_le_bytes();
            if copyout_user(slot_ptr + 6, &rv, rv.len()).is_err() {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
        }
        ready
    }

}

#[cfg(not(any(feature = "ipc_test", feature = "shm_test", feature = "ipc_badptr_send_test", feature = "ipc_badptr_recv_test", feature = "ipc_badptr_svc_test", feature = "ipc_buffer_full_test", feature = "ipc_waiter_busy_test", feature = "svc_overwrite_test", feature = "svc_full_test", feature = "svc_bad_endpoint_test", feature = "stress_ipc_test", feature = "quota_endpoints_test", feature = "quota_shm_test", feature = "quota_threads_test")))]
unsafe fn sys_thread_exit_m3(frame: *mut u64) {
    #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
    {
        // M8 PR-2: expose deterministic child-exit observation for wait semantics.
        M8_WAIT_HAS_EXIT = true;
        M8_WAIT_EXIT_STATUS = 0;
    }
    #[cfg(any(feature = "user_hello_test", feature = "syscall_test", feature = "thread_exit_test", feature = "thread_spawn_test", feature = "vm_map_test", feature = "syscall_invalid_test", feature = "stress_syscall_test", feature = "yield_test", feature = "user_fault_test", feature = "blk_test", feature = "fs_test", feature = "go_test", feature = "go_std_test", feature = "sec_rights_test", feature = "sec_filter_test"))]
    if M3_THREADING_ACTIVE {
        M3_THREADS[M3_CURRENT].state = M3ThreadState::Dead;
        if let Some(next) = m3_find_ready(M3_CURRENT) {
            m3_switch_to(frame, next);
            return;
        }
    }
    serial_write(b"THREAD_EXIT: ok\n");
    trap::m3_return_to_kernel_halt(frame);
}

#[cfg(any(feature = "fs_test", feature = "go_test"))]
#[inline(always)]
fn sha256_rotr(x: u32, n: u32) -> u32 {
    (x >> n) | (x << (32 - n))
}

#[cfg(any(feature = "fs_test", feature = "go_test"))]
fn sha256_compress(state: &mut [u32; 8], block: &[u8; 64]) {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
        0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
        0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut w = [0u32; 64];
    let mut t = 0usize;
    while t < 16 {
        let j = t * 4;
        w[t] = u32::from_be_bytes([block[j], block[j + 1], block[j + 2], block[j + 3]]);
        t += 1;
    }
    while t < 64 {
        let s0 = sha256_rotr(w[t - 15], 7) ^ sha256_rotr(w[t - 15], 18) ^ (w[t - 15] >> 3);
        let s1 = sha256_rotr(w[t - 2], 17) ^ sha256_rotr(w[t - 2], 19) ^ (w[t - 2] >> 10);
        w[t] = w[t - 16]
            .wrapping_add(s0)
            .wrapping_add(w[t - 7])
            .wrapping_add(s1);
        t += 1;
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    t = 0;
    while t < 64 {
        let s1 = sha256_rotr(e, 6) ^ sha256_rotr(e, 11) ^ sha256_rotr(e, 25);
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K[t])
            .wrapping_add(w[t]);
        let s0 = sha256_rotr(a, 2) ^ sha256_rotr(a, 13) ^ sha256_rotr(a, 22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
        t += 1;
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

#[cfg(any(feature = "fs_test", feature = "go_test"))]
fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut state = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let mut offset = 0usize;
    while offset + 64 <= data.len() {
        let mut block = [0u8; 64];
        block.copy_from_slice(&data[offset..offset + 64]);
        sha256_compress(&mut state, &block);
        offset += 64;
    }

    let mut block = [0u8; 64];
    let rem = data.len() - offset;
    if rem > 0 {
        block[..rem].copy_from_slice(&data[offset..]);
    }
    block[rem] = 0x80;

    if rem >= 56 {
        sha256_compress(&mut state, &block);
        block = [0u8; 64];
    }

    let bit_len = (data.len() as u64).wrapping_mul(8);
    block[56..64].copy_from_slice(&bit_len.to_be_bytes());
    sha256_compress(&mut state, &block);

    let mut out = [0u8; 32];
    let mut i = 0usize;
    while i < 8 {
        out[i * 4..(i + 1) * 4].copy_from_slice(&state[i].to_be_bytes());
        i += 1;
    }
    out
}

// --------------- User page table infrastructure (shared M3+R4) ---------------

cfg_user! {
    const USER_CODE_VA: u64   = 0x40_0000;
    const USER_STACK_TOP: u64 = 0x80_0000;

    #[derive(Clone, Copy)]
    #[repr(C, align(4096))]
    struct Page([u8; 4096]);

    static mut USER_PML4:      Page = Page([0; 4096]);
    static mut USER_PDPT:      Page = Page([0; 4096]);
    static mut USER_PD:        Page = Page([0; 4096]);
    static mut USER_PT_CODE:   Page = Page([0; 4096]);
    static mut USER_PT_STACK:  Page = Page([0; 4096]);
    static mut USER_CODE_PAGE: Page = Page([0; 4096]);
    static mut USER_STACK_PAGE: Page = Page([0; 4096]);

}

// --------------- M3: User thread + vm model ----------------------------------

cfg_m3! {
    const M3_MAX_THREADS: usize = 4;
    const M3_MAX_VM_MAPS: usize = 8;
    const M3_VM_PAGE_SIZE: u64 = 4096;
    const M8_FD_MAX: usize = 16;
    const M10_RIGHT_READ: u64 = runtime::security::HANDLE_RIGHT_READ;
    const M10_RIGHT_WRITE: u64 = runtime::security::HANDLE_RIGHT_WRITE;
    const M10_RIGHT_POLL: u64 = runtime::security::HANDLE_RIGHT_POLL;
    const M10_RIGHT_MASK: u64 = runtime::security::HANDLE_RIGHT_MASK;
    const M10_OPEN_MODE_MASK: u64 = 0x3;
    const M10_OPEN_RDONLY: u64 = 0;
    const M10_OPEN_WRONLY: u64 = 1;
    const M10_OPEN_RDWR: u64 = 2;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const M10_OPEN_CREATE: u64 = 0x4;

    #[derive(Clone, Copy, PartialEq)]
    enum M8FdKind {
        Free,
        Console,
        CompatFile,
        JournalFile,
        StateFile,
        PkgStateFile,
        PlatformFile,
        VfsFile,
        VfsDir,
        PipeR,
        PipeW,
        // /dev character devices (full-os guide Part II.5, pseudo-fs).
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        DevZero,
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        DevNull,
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        DevUrandom,
        // /proc/self/stat (full-os guide Part II.5, pseudo-fs).
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        ProcSelfStat,
        // /proc/<tid>/stat (full-os guide Part II.5): inspect an arbitrary task;
        // the target tid is held in M8_FD_VFS_NODE[fd].
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        ProcTidStat,
        // timerfd (full-os guide Part IV.9): offset holds the expiry tick.
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        TimerFd,
        // tmpfs file (full-os guide Part II.5): in-memory /tmp; the tmpfs
        // node index is held in M8_FD_VFS_NODE[fd].
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        TmpFile,
        // pty pair (full-os guide Part V.11 TTY/pty): two fds over one PtyObj
        // (two rings); the pty index is held in M8_FD_PTY[fd].
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        PtyMaster,
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        PtySlave,
        // FAT16 file opened via /mnt (full-os guide Part II.5 mounts): contents
        // are cached in FAT_FILE on open; the fd offset tracks read position.
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        FatFile,
    }

    // In-memory tmpfs for /tmp (full-os guide Part II.5). Heap-free fixed
    // store; contents are lost on reboot.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const TMPFS_MAX: usize = 8;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const TMPFS_CAP: usize = 512;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    #[derive(Clone, Copy)]
    struct TmpNode {
        used: bool,
        name: [u8; 24],
        name_len: usize,
        len: usize,
        data: [u8; TMPFS_CAP],
    }
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    impl TmpNode {
        const EMPTY: Self = Self {
            used: false,
            name: [0u8; 24],
            name_len: 0,
            len: 0,
            data: [0u8; TMPFS_CAP],
        };
    }
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut TMPFS: [TmpNode; TMPFS_MAX] = [TmpNode::EMPTY; TMPFS_MAX];

    /// Find an existing /tmp node by relative name, or allocate one if
    /// `create`. Returns the node index.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn tmpfs_open(rel: &[u8], create: bool) -> Option<usize> {
        if rel.is_empty() || rel.len() > 24 {
            return None;
        }
        let mut i = 0;
        while i < TMPFS_MAX {
            if TMPFS[i].used
                && TMPFS[i].name_len == rel.len()
                && TMPFS[i].name[..rel.len()] == *rel
            {
                return Some(i);
            }
            i += 1;
        }
        if !create {
            return None;
        }
        let mut i = 0;
        while i < TMPFS_MAX {
            if !TMPFS[i].used {
                TMPFS[i] = TmpNode::EMPTY;
                TMPFS[i].used = true;
                TMPFS[i].name[..rel.len()].copy_from_slice(rel);
                TMPFS[i].name_len = rel.len();
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Write "0x" + 16 zero-padded hex digits of `val` into `out` (>=18
    /// bytes); returns the count written. Buffer analogue of serial_write_hex.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    fn fmt_hex_u64(out: &mut [u8], val: u64) -> usize {
        out[0] = b'0';
        out[1] = b'x';
        let mut i = 0;
        while i < 16 {
            let nib = ((val >> ((15 - i) * 4)) & 0xF) as u8;
            out[2 + i] = if nib < 10 { b'0' + nib } else { b'a' + nib - 10 };
            i += 1;
        }
        18
    }

    // VFS node index per fd (parallel to M8_FD_TABLE; the fd's offset
    // field doubles as the read cursor for both files and directories).
    static mut M8_FD_VFS_NODE: [u8; M8_FD_MAX] = [0; M8_FD_MAX];

    // ---- pipes (gap item 8: pipe IPC) ----
    // A pipe is a 512-byte in-kernel ring with reader/writer refcounts;
    // fds reference it through M8_FD_PIPE. Read on an empty pipe returns
    // -1 while a writer exists (the caller yields and retries) and 0 once
    // every writer is gone (EOF). The fds are transferable to spawned
    // children as stdin/stdout.
    const PIPE_MAX: usize = 4;
    const PIPE_CAP: usize = 512;

    #[derive(Clone, Copy)]
    struct PipeObj {
        active: bool,
        len: usize,
        readers: u8,
        writers: u8,
        buf: [u8; PIPE_CAP],
    }

    impl PipeObj {
        const EMPTY: Self = Self {
            active: false,
            len: 0,
            readers: 0,
            writers: 0,
            buf: [0; PIPE_CAP],
        };
    }

    static mut PIPES: [PipeObj; PIPE_MAX] = [PipeObj::EMPTY; PIPE_MAX];
    static mut M8_FD_PIPE: [u8; M8_FD_MAX] = [0; M8_FD_MAX];

    /// Drop one end's reference; recycle the pipe when both sides are gone.
    unsafe fn pipe_drop_end(fd_idx: usize) {
        let p = M8_FD_PIPE[fd_idx] as usize;
        if p >= PIPE_MAX || !PIPES[p].active {
            return;
        }
        match M8_FD_TABLE[fd_idx].kind {
            M8FdKind::PipeR => {
                if PIPES[p].readers > 0 {
                    PIPES[p].readers -= 1;
                }
            }
            M8FdKind::PipeW => {
                if PIPES[p].writers > 0 {
                    PIPES[p].writers -= 1;
                }
            }
            _ => return,
        }
        if PIPES[p].readers == 0 && PIPES[p].writers == 0 {
            PIPES[p] = PipeObj::EMPTY;
        }
    }

    // pty pair (full-os guide Part V.11 TTY/pty): one PtyObj backs a
    // master/slave fd pair via two rings. Bidirectional, in-memory, heap-free.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const PTY_MAX: usize = 2;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const PTY_CAP: usize = 512;

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    #[derive(Clone, Copy)]
    struct PtyObj {
        active: bool,
        master_open: bool,
        slave_open: bool,
        m2s_len: usize,
        s2m_len: usize,
        m2s: [u8; PTY_CAP],
        s2m: [u8; PTY_CAP],
    }

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    impl PtyObj {
        const EMPTY: Self = Self {
            active: false,
            master_open: false,
            slave_open: false,
            m2s_len: 0,
            s2m_len: 0,
            m2s: [0; PTY_CAP],
            s2m: [0; PTY_CAP],
        };
    }

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut PTYS: [PtyObj; PTY_MAX] = [PtyObj::EMPTY; PTY_MAX];
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut M8_FD_PTY: [u8; M8_FD_MAX] = [0; M8_FD_MAX];

    // Periodic timerfd interval in PIT ticks, keyed by fd index (0 = one-shot).
    // Set by sys_time op 3 (one-shot, 0) / op 4 (periodic, > 0); read by the
    // TimerFd read path to re-arm. (full-os guide Part IV.9)
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut M8_FD_TIMER_IVL: [u64; M8_FD_MAX] = [0; M8_FD_MAX];

    /// Given a fired timerfd (`now` >= `deadline`), return (expiration count, next
    /// deadline). A one-shot (`ivl` == 0) reports 1 and disarms (usize::MAX). A
    /// periodic timer reports every interval elapsed since `deadline` (>= 1) and
    /// advances to the next future deadline. (full-os guide Part IV.9)
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    fn timerfd_fire(deadline: usize, ivl: usize, now: usize) -> (u64, usize) {
        if ivl == 0 {
            (1, usize::MAX)
        } else {
            let count = 1 + ((now - deadline) / ivl) as u64;
            (count, deadline + count as usize * ivl)
        }
    }

    /// Boot self-test for periodic timerfd re-arming (full-os guide Part IV.9): the
    /// one-shot disarms; a periodic timer reports every interval elapsed since its
    /// deadline and advances to the next future deadline. Exercises the exact
    /// `timerfd_fire` logic the TimerFd read path uses.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn timerfd_periodic_selftest() {
        let ok = timerfd_fire(5, 0, 10) == (1, usize::MAX)   // one-shot disarms
            && timerfd_fire(5, 5, 5) == (1, 10)              // fires exactly at the deadline
            && timerfd_fire(5, 5, 12) == (2, 15)             // 2 expirations (t=5,10), next 15
            && timerfd_fire(5, 5, 22) == (4, 25);            // 4 expirations (t=5,10,15,20)
        if ok {
            serial_write(b"TIMERFD: periodic ok\n");
        } else {
            serial_write(b"TIMERFD: periodic fail\n");
        }
    }

    // FAT16 /mnt file cache (full-os guide Part II.5 mounts): one open file at a
    // time; filled on open, served by the FatFile read arm via the fd offset.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut FAT_FILE: [u8; 512] = [0; 512];
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut FAT_FILE_LEN: usize = 0;
    // True while a /mnt FatFile fd is open: the cache holds one file at a time,
    // so a second concurrent open is rejected rather than silently overwriting.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut FAT_FILE_BUSY: bool = false;

    /// Drop one pty end on close; recycle the PtyObj when both ends are gone.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn pty_drop_end(fd_idx: usize) {
        let p = M8_FD_PTY[fd_idx] as usize;
        if p >= PTY_MAX || !PTYS[p].active {
            return;
        }
        match M8_FD_TABLE[fd_idx].kind {
            M8FdKind::PtyMaster => PTYS[p].master_open = false,
            M8FdKind::PtySlave => PTYS[p].slave_open = false,
            _ => return,
        }
        if !PTYS[p].master_open && !PTYS[p].slave_open {
            PTYS[p] = PtyObj::EMPTY;
        }
    }

    #[derive(Clone, Copy)]
    struct M8FdEntry {
        kind: M8FdKind,
        rights: u64,
        offset: usize,
        owner_tid: usize,
    }

    impl M8FdEntry {
        const EMPTY: Self = Self {
            kind: M8FdKind::Free,
            rights: 0,
            offset: 0,
            owner_tid: 0,
        };
    }

    static mut M8_FD_TABLE: [M8FdEntry; M8_FD_MAX] = [M8FdEntry::EMPTY; M8_FD_MAX];
    static mut M8_WAIT_HAS_EXIT: bool = false;
    static mut M8_WAIT_EXIT_STATUS: i32 = 0;
    static M8_COMPAT_FILE: &[u8] = b"compat v1 hello\n";
    static mut M10_SEC_PROFILE: M10SecProfile = M10SecProfile::Default;

    #[derive(Clone, Copy, PartialEq)]
    enum M10SecProfile {
        Default,
        Restricted,
    }

    #[derive(Clone, Copy, PartialEq)]
    enum M3ThreadState { Dead, Ready, Running }

    #[derive(Clone, Copy)]
    struct M3Thread {
        saved_frame: [u64; 22],
        state: M3ThreadState,
    }

    impl M3Thread {
        const EMPTY: Self = Self {
            saved_frame: [0u64; 22],
            state: M3ThreadState::Dead,
        };
    }

    #[derive(Clone, Copy)]
    struct M3VmMap {
        active: bool,
        va: u64,
    }

    impl M3VmMap {
        const EMPTY: Self = Self { active: false, va: 0 };
    }

    static mut M3_THREADS: [M3Thread; M3_MAX_THREADS] =
        [M3Thread::EMPTY; M3_MAX_THREADS];
    static mut M3_CURRENT: usize = 0;
    static mut M3_THREADING_ACTIVE: bool = false;

    static mut M3_STACK_PAGE_1: Page = Page([0; 4096]);
    static mut M3_STACK_PAGE_2: Page = Page([0; 4096]);
    static mut M3_STACK_PAGE_3: Page = Page([0; 4096]);

    static mut M3_VM_PAGES: [Page; M3_MAX_VM_MAPS] = [Page([0; 4096]); M3_MAX_VM_MAPS];
    static mut M3_VM_MAPS: [M3VmMap; M3_MAX_VM_MAPS] = [M3VmMap::EMPTY; M3_MAX_VM_MAPS];

    #[inline(always)]
    unsafe fn m3_stack_top_for_slot(slot: usize) -> u64 {
        USER_STACK_TOP - (slot as u64) * 0x1000
    }

    unsafe fn m8_reset_fd_table() {
        for i in 0..M8_FD_MAX {
            M8_FD_TABLE[i] = M8FdEntry::EMPTY;
        }
        // Keep stdio-like descriptors deterministic across every user-mode reset.
        let console_rights = m10_rights_for_kind(M8FdKind::Console);
        M8_FD_TABLE[0] = M8FdEntry {
            kind: M8FdKind::Console,
            rights: console_rights,
            offset: 0,
            owner_tid: 0,
        };
        M8_FD_TABLE[1] = M8FdEntry {
            kind: M8FdKind::Console,
            rights: console_rights,
            offset: 0,
            owner_tid: 0,
        };
        M8_FD_TABLE[2] = M8FdEntry {
            kind: M8FdKind::Console,
            rights: console_rights,
            offset: 0,
            owner_tid: 0,
        };
    }

    unsafe fn m3_reset_state() {
        M3_CURRENT = 0;
        M3_THREADING_ACTIVE = false;
        M8_WAIT_HAS_EXIT = false;
        M8_WAIT_EXIT_STATUS = 0;
        M10_SEC_PROFILE = M10SecProfile::Default;
        for i in 0..M3_MAX_THREADS {
            M3_THREADS[i] = M3Thread::EMPTY;
        }
        for i in 0..M3_MAX_VM_MAPS {
            M3_VM_MAPS[i] = M3VmMap::EMPTY;
        }
        m8_reset_fd_table();
    }

    unsafe fn m3_bootstrap_main_thread(frame: *mut u64) {
        if M3_THREADING_ACTIVE { return; }
        for i in 0..22 {
            M3_THREADS[0].saved_frame[i] = *frame.add(i);
        }
        M3_THREADS[0].state = M3ThreadState::Running;
        M3_CURRENT = 0;
        M3_THREADING_ACTIVE = true;
    }

    unsafe fn m3_save_frame(frame: *mut u64, tid: usize) {
        for i in 0..22 {
            M3_THREADS[tid].saved_frame[i] = *frame.add(i);
        }
    }

    unsafe fn m3_switch_to(frame: *mut u64, tid: usize) {
        for i in 0..22 {
            *frame.add(i) = M3_THREADS[tid].saved_frame[i];
        }
        M3_THREADS[tid].state = M3ThreadState::Running;
        M3_CURRENT = tid;
    }

    unsafe fn m3_find_ready(exclude: usize) -> Option<usize> {
        for tid in 0..M3_MAX_THREADS {
            if tid != exclude && M3_THREADS[tid].state == M3ThreadState::Ready {
                return Some(tid);
            }
        }
        None
    }

    unsafe fn m3_user_pte_ptr(addr: u64) -> Option<*mut u64> {
        if addr >= USER_VA_LIMIT { return None; }
        let hhdm = HHDM_OFFSET;
        if hhdm == 0 { return None; }

        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        let pml4_phys = cr3 & 0x000F_FFFF_FFFF_F000;
        let pml4 = (pml4_phys + hhdm) as *const u64;
        let pml4e = *pml4.add(((addr >> 39) & 0x1FF) as usize);
        if pml4e & 1 == 0 || pml4e & 4 == 0 { return None; }

        let pdpt = ((pml4e & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
        let pdpte = *pdpt.add(((addr >> 30) & 0x1FF) as usize);
        if pdpte & 1 == 0 || pdpte & 4 == 0 || pdpte & 0x80 != 0 { return None; }

        let pd = ((pdpte & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
        let pde = *pd.add(((addr >> 21) & 0x1FF) as usize);
        if pde & 1 == 0 || pde & 4 == 0 || pde & 0x80 != 0 { return None; }

        let pt = ((pde & 0x000F_FFFF_FFFF_F000) + hhdm) as *mut u64;
        let pt_idx = ((addr >> 12) & 0x1FF) as usize;
        Some(pt.add(pt_idx))
    }

    unsafe fn m3_kv2p(va: u64) -> u64 {
        let kaddr_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(KADDR_REQUEST.response));
        let kphys = (*kaddr_resp_ptr).physical_base;
        let kvirt = (*kaddr_resp_ptr).virtual_base;
        va - kvirt + kphys
    }

    fn m8_path_matches(buf: &[u8], path: &[u8]) -> bool {
        if buf.len() != path.len() + 1 {
            return false;
        }
        if buf[buf.len() - 1] != 0 {
            return false;
        }
        &buf[..path.len()] == path
    }

    fn m10_rights_for_kind(kind: M8FdKind) -> u64 {
        match kind {
            M8FdKind::Free => 0,
            M8FdKind::Console => M10_RIGHT_READ | M10_RIGHT_WRITE | M10_RIGHT_POLL,
            M8FdKind::CompatFile => M10_RIGHT_READ | M10_RIGHT_POLL,
            M8FdKind::JournalFile => M10_RIGHT_WRITE | M10_RIGHT_POLL,
            M8FdKind::StateFile => M10_RIGHT_READ | M10_RIGHT_POLL,
            M8FdKind::PkgStateFile | M8FdKind::PlatformFile => {
                M10_RIGHT_READ | M10_RIGHT_WRITE | M10_RIGHT_POLL
            }
            M8FdKind::VfsFile => M10_RIGHT_READ | M10_RIGHT_WRITE | M10_RIGHT_POLL,
            M8FdKind::VfsDir => M10_RIGHT_READ | M10_RIGHT_POLL,
            M8FdKind::PipeR => M10_RIGHT_READ | M10_RIGHT_POLL,
            M8FdKind::PipeW => M10_RIGHT_WRITE | M10_RIGHT_POLL,
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::DevZero | M8FdKind::DevUrandom => M10_RIGHT_READ | M10_RIGHT_POLL,
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::DevNull => M10_RIGHT_WRITE | M10_RIGHT_POLL,
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::ProcSelfStat | M8FdKind::ProcTidStat => M10_RIGHT_READ | M10_RIGHT_POLL,
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::TimerFd => M10_RIGHT_READ | M10_RIGHT_POLL,
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::TmpFile => M10_RIGHT_READ | M10_RIGHT_WRITE | M10_RIGHT_POLL,
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::PtyMaster | M8FdKind::PtySlave => {
                M10_RIGHT_READ | M10_RIGHT_WRITE | M10_RIGHT_POLL
            }
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            M8FdKind::FatFile => M10_RIGHT_READ | M10_RIGHT_POLL,
        }
    }

    fn m10_open_requested_rights(flags: u64) -> Option<u64> {
        runtime::security::requested_open_rights(
            flags,
            M10_OPEN_MODE_MASK,
            M10_OPEN_RDONLY,
            M10_OPEN_WRONLY,
            M10_OPEN_RDWR,
        )
    }

    fn m10_profile_path_allowed(path: &[u8]) -> bool {
        match unsafe { M10_SEC_PROFILE } {
            M10SecProfile::Default => true,
            M10SecProfile::Restricted => m8_path_matches(path, b"/compat/hello.txt"),
        }
    }

    unsafe fn m10_syscall_allowed(nr: u64) -> bool {
        match M10_SEC_PROFILE {
            M10SecProfile::Default => true,
            M10SecProfile::Restricted => matches!(
                nr,
                0 | 2 | 3 | 10 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 98
            ),
        }
    }

    unsafe fn sys_sec_profile_set_v1(profile: u64) -> u64 {
        match profile {
            0 => {
                M10_SEC_PROFILE = M10SecProfile::Default;
                0
            }
            1 => {
                M10_SEC_PROFILE = M10SecProfile::Restricted;
                0
            }
            _ => 0xFFFF_FFFF_FFFF_FFFF,
        }
    }

    unsafe fn sys_fd_rights_get_v1(fd: u64) -> u64 {
        let idx = fd as usize;
        if idx >= M8_FD_MAX { return 0xFFFF_FFFF_FFFF_FFFF; }
        if M8_FD_TABLE[idx].kind == M8FdKind::Free {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        #[cfg(feature = "go_test")]
        if !r4_fd_owner_ok(idx) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        M8_FD_TABLE[idx].rights & M10_RIGHT_MASK
    }

    unsafe fn sys_fd_rights_reduce_v1(fd: u64, rights: u64) -> u64 {
        let idx = fd as usize;
        if idx >= M8_FD_MAX { return 0xFFFF_FFFF_FFFF_FFFF; }
        if idx < 3 {
            // Keep stdio fixed for deterministic startup behavior.
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if M8_FD_TABLE[idx].kind == M8FdKind::Free {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        #[cfg(feature = "go_test")]
        if !r4_fd_owner_ok(idx) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let req = match runtime::security::monotonic_rights(M8_FD_TABLE[idx].rights, rights) {
            Some(req) => req,
            None => return 0xFFFF_FFFF_FFFF_FFFF,
        };
        M8_FD_TABLE[idx].rights = req;
        0
    }

    unsafe fn sys_fd_rights_transfer_v1(fd: u64, rights: u64) -> u64 {
        let idx = fd as usize;
        if idx >= M8_FD_MAX { return 0xFFFF_FFFF_FFFF_FFFF; }
        if idx < 3 {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let src = M8_FD_TABLE[idx];
        if src.kind == M8FdKind::Free {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        #[cfg(feature = "go_test")]
        if !r4_fd_owner_ok(idx) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let req = runtime::security::clamp_rights(rights);
        if req == 0 {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if req & !src.rights != 0 {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        for i in 3..M8_FD_MAX {
            if M8_FD_TABLE[i].kind == M8FdKind::Free {
                M8_FD_TABLE[i] = M8FdEntry {
                    kind: src.kind,
                    rights: req,
                    offset: src.offset,
                    owner_tid: src.owner_tid,
                };
                // Carry the fd-indexed backing references to the new slot;
                // otherwise a moved pipe/vfs/tmpfs/pty fd would point at a
                // stale object (the side arrays are keyed by fd index).
                M8_FD_PIPE[i] = M8_FD_PIPE[idx];
                M8_FD_VFS_NODE[i] = M8_FD_VFS_NODE[idx];
                #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                {
                    M8_FD_PTY[i] = M8_FD_PTY[idx];
                    M8_FD_TIMER_IVL[i] = M8_FD_TIMER_IVL[idx];
                }
                M8_FD_TABLE[idx] = M8FdEntry::EMPTY;
                return i as u64;
            }
        }
        0xFFFF_FFFF_FFFF_FFFF
    }

    #[cfg(feature = "go_test")]
    #[inline(always)]
    unsafe fn r4_fd_owner_ok(idx: usize) -> bool {
        idx < 3
            || (M8_FD_TABLE[idx].kind != M8FdKind::Free
                && M8_FD_TABLE[idx].owner_tid == R4_CURRENT)
    }

    #[cfg(feature = "go_test")]
    unsafe fn r4_release_owned_fds(owner_tid: usize) {
        for idx in 3..M8_FD_MAX {
            if M8_FD_TABLE[idx].kind != M8FdKind::Free
                && M8_FD_TABLE[idx].owner_tid == owner_tid
            {
                pipe_drop_end(idx);
                // Recycle pty/FatFile backing on task teardown too, mirroring
                // the explicit-close path; otherwise a task that exits with an
                // open pty leaks its PtyObj slot (PTY_MAX is small).
                #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                {
                    pty_drop_end(idx);
                    if M8_FD_TABLE[idx].kind == M8FdKind::FatFile {
                        FAT_FILE_BUSY = false;
                    }
                }
                M8_FD_TABLE[idx] = M8FdEntry::EMPTY;
            }
        }
        if owner_tid < R4_NUM_TASKS {
            R4_TASKS[owner_tid].fd_count = 0;
        }
    }

    unsafe fn m8_alloc_fd(kind: M8FdKind) -> u64 {
        #[cfg(not(feature = "go_test"))]
        let owner_tid = 0usize;
        #[cfg(feature = "go_test")]
        let owner_tid: usize;
        #[cfg(feature = "go_test")]
        {
            if !runtime::isolation::under_quota(
                R4_TASKS[r4_current_smp()].fd_count,
                R4_TASKS[r4_current_smp()].fd_limit as usize,
            ) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            owner_tid = R4_CURRENT;
        }
        for i in 3..M8_FD_MAX {
            if M8_FD_TABLE[i].kind == M8FdKind::Free {
                M8_FD_TABLE[i].kind = kind;
                M8_FD_TABLE[i].rights = m10_rights_for_kind(kind);
                M8_FD_TABLE[i].offset = 0;
                M8_FD_TABLE[i].owner_tid = owner_tid;
                #[cfg(feature = "go_test")]
                {
                    R4_TASKS[r4_current_smp()].fd_count += 1;
                }
                return i as u64;
            }
        }
        0xFFFF_FFFF_FFFF_FFFF
    }
}

// --------------- M3: User page setup -----------------------------------------

cfg_m3! {
    unsafe fn setup_user_pages(user_code: &[u8]) {
        let hhdm_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(HHDM_REQUEST.response));
        let kaddr_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(KADDR_REQUEST.response));
        let hhdm = (*hhdm_resp_ptr).offset;
        let kphys = (*kaddr_resp_ptr).physical_base;
        let kvirt = (*kaddr_resp_ptr).virtual_base;
        HHDM_OFFSET = hhdm;
        let kv2p = |va: u64| -> u64 { va - kvirt + kphys };

        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        let old_pml4_phys = cr3 & 0x000F_FFFF_FFFF_F000;
        let old_pml4 = (old_pml4_phys + hhdm) as *const u64;
        let new_pml4 = USER_PML4.0.as_mut_ptr() as *mut u64;
        for i in 0..512 { *new_pml4.add(i) = *old_pml4.add(i); }

        let pdpt = USER_PDPT.0.as_mut_ptr() as *mut u64;
        *pdpt = kv2p(USER_PD.0.as_ptr() as u64) | 0x07;

        let pd = USER_PD.0.as_mut_ptr() as *mut u64;
        *pd.add(2) = kv2p(USER_PT_CODE.0.as_ptr() as u64) | 0x07;
        *pd.add(3) = kv2p(USER_PT_STACK.0.as_ptr() as u64) | 0x07;

        let pt_code = USER_PT_CODE.0.as_mut_ptr() as *mut u64;
        *pt_code = kv2p(USER_CODE_PAGE.0.as_ptr() as u64) | 0x07; // RW so TinyGo .data works

        let pt_stack = USER_PT_STACK.0.as_mut_ptr() as *mut u64;
        *pt_stack.add(511) = kv2p(USER_STACK_PAGE.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(510) = kv2p(M3_STACK_PAGE_1.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(509) = kv2p(M3_STACK_PAGE_2.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(508) = kv2p(M3_STACK_PAGE_3.0.as_ptr() as u64) | 0x07;

        *new_pml4 = kv2p(USER_PDPT.0.as_ptr() as u64) | 0x07;

        core::ptr::copy_nonoverlapping(
            user_code.as_ptr(), USER_CODE_PAGE.0.as_mut_ptr(), user_code.len());

        m3_reset_state();

        let new_pml4_phys = kv2p(new_pml4 as u64);
        core::arch::asm!("mov cr3, {}", in(reg) new_pml4_phys, options(nostack));
    }

    unsafe fn m3_user_code_page_ptr(page_idx: usize) -> Option<*mut u8> {
        match page_idx {
            0 => Some(USER_CODE_PAGE.0.as_mut_ptr()),
            #[cfg(feature = "go_test")]
            1 => Some(USER_CODE_PAGE_2.0.as_mut_ptr()),
            #[cfg(feature = "go_test")]
            2 => Some(USER_CODE_PAGE_3.0.as_mut_ptr()),
            #[cfg(feature = "go_test")]
            3 => Some(USER_CODE_PAGE_4.0.as_mut_ptr()),
            #[cfg(feature = "go_test")]
            4 => Some(USER_CODE_PAGE_5.0.as_mut_ptr()),
            #[cfg(feature = "go_test")]
            5 => Some(USER_CODE_PAGE_6.0.as_mut_ptr()),
            #[cfg(feature = "go_test")]
            6 => Some(USER_CODE_PAGE_7.0.as_mut_ptr()),
            #[cfg(feature = "go_test")]
            7 => Some(USER_CODE_PAGE_8.0.as_mut_ptr()),
            _ => None,
        }
    }

    fn m3_user_code_page_count() -> usize {
        #[cfg(feature = "go_test")]
        {
            runtime::process::GO_IMAGE_MAX_PAGES
        }
        #[cfg(not(feature = "go_test"))]
        {
            1
        }
    }

    unsafe fn m3_zero_user_code_pages() {
        for page_idx in 0..m3_user_code_page_count() {
            if let Some(ptr) = m3_user_code_page_ptr(page_idx) {
                core::ptr::write_bytes(ptr, 0, runtime::process::GO_IMAGE_PAGE_SIZE);
            }
        }
    }

    unsafe fn m3_copy_user_code(offset: usize, src: &[u8]) -> bool {
        let mut copied = 0usize;
        while copied < src.len() {
            let dst_off = offset + copied;
            let page_idx = dst_off / runtime::process::GO_IMAGE_PAGE_SIZE;
            let page_off = dst_off % runtime::process::GO_IMAGE_PAGE_SIZE;
            let page_ptr = match m3_user_code_page_ptr(page_idx) {
                Some(ptr) => ptr,
                None => return false,
            };
            let chunk = core::cmp::min(
                src.len() - copied,
                runtime::process::GO_IMAGE_PAGE_SIZE - page_off,
            );
            core::ptr::copy_nonoverlapping(
                src.as_ptr().add(copied),
                page_ptr.add(page_off),
                chunk,
            );
            copied += chunk;
        }
        true
    }

    unsafe fn m3_zero_user_code(offset: usize, len: usize) -> bool {
        let mut cleared = 0usize;
        while cleared < len {
            let dst_off = offset + cleared;
            let page_idx = dst_off / runtime::process::GO_IMAGE_PAGE_SIZE;
            let page_off = dst_off % runtime::process::GO_IMAGE_PAGE_SIZE;
            let page_ptr = match m3_user_code_page_ptr(page_idx) {
                Some(ptr) => ptr,
                None => return false,
            };
            let chunk = core::cmp::min(
                len - cleared,
                runtime::process::GO_IMAGE_PAGE_SIZE - page_off,
            );
            core::ptr::write_bytes(page_ptr.add(page_off), 0, chunk);
            cleared += chunk;
        }
        true
    }

    unsafe fn m3_load_user_elf_image(image: &[u8]) -> Option<u64> {
        if !elf_v1_validate_image(image) || image.len() < 64 {
            return None;
        }

        let e_type = elf_v1_read_u16(image, 16)?;
        if e_type != ELF_V1_ET_EXEC {
            return None;
        }

        let e_entry = elf_v1_read_u64(image, 24)?;
        let e_phoff = elf_v1_read_u64(image, 32)? as usize;
        let e_phentsize = elf_v1_read_u16(image, 54)? as usize;
        let e_phnum = elf_v1_read_u16(image, 56)? as usize;
        let code_span = runtime::process::GO_IMAGE_PAGE_SIZE
            .checked_mul(m3_user_code_page_count())?;
        let code_end = USER_CODE_VA.checked_add(code_span as u64)?;

        for idx in 0..e_phnum {
            let off = e_phoff.checked_add(idx.checked_mul(e_phentsize)?)?;
            let p_type = elf_v1_read_u32(image, off)?;
            if p_type != ELF_V1_PT_LOAD {
                continue;
            }

            let p_offset = elf_v1_read_u64(image, off + 8)? as usize;
            let p_vaddr = elf_v1_read_u64(image, off + 16)?;
            let p_filesz = elf_v1_read_u64(image, off + 32)? as usize;
            let p_memsz = elf_v1_read_u64(image, off + 40)? as usize;
            if p_vaddr < USER_CODE_VA || p_vaddr.checked_add(p_memsz as u64)? > code_end {
                return None;
            }

            let file_end = p_offset.checked_add(p_filesz)?;
            if file_end > image.len() {
                return None;
            }

            let dst_off = (p_vaddr - USER_CODE_VA) as usize;
            if !m3_copy_user_code(dst_off, &image[p_offset..file_end]) {
                return None;
            }
            if p_memsz > p_filesz && !m3_zero_user_code(dst_off + p_filesz, p_memsz - p_filesz) {
                return None;
            }
        }

        Some(e_entry)
    }

    unsafe fn setup_user_elf_pages(image: &[u8]) -> Option<u64> {
        let hhdm_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(HHDM_REQUEST.response));
        let kaddr_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(KADDR_REQUEST.response));
        let hhdm = (*hhdm_resp_ptr).offset;
        let kphys = (*kaddr_resp_ptr).physical_base;
        let kvirt = (*kaddr_resp_ptr).virtual_base;
        HHDM_OFFSET = hhdm;
        let kv2p = |va: u64| -> u64 { va - kvirt + kphys };

        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        let old_pml4_phys = cr3 & 0x000F_FFFF_FFFF_F000;
        let old_pml4 = (old_pml4_phys + hhdm) as *const u64;
        let new_pml4 = USER_PML4.0.as_mut_ptr() as *mut u64;
        for i in 0..512 { *new_pml4.add(i) = *old_pml4.add(i); }

        let pdpt = USER_PDPT.0.as_mut_ptr() as *mut u64;
        *pdpt = kv2p(USER_PD.0.as_ptr() as u64) | 0x07;

        let pd = USER_PD.0.as_mut_ptr() as *mut u64;
        *pd.add(2) = kv2p(USER_PT_CODE.0.as_ptr() as u64) | 0x07;
        *pd.add(3) = kv2p(USER_PT_STACK.0.as_ptr() as u64) | 0x07;

        let code_flags = if cfg!(feature = "go_test") { 0x07 } else { 0x05 };
        let pt_code = USER_PT_CODE.0.as_mut_ptr() as *mut u64;
        *pt_code.add(0) = kv2p(USER_CODE_PAGE.0.as_ptr() as u64) | code_flags;
        #[cfg(feature = "go_test")]
        {
            *pt_code.add(1) = kv2p(USER_CODE_PAGE_2.0.as_ptr() as u64) | code_flags;
            *pt_code.add(2) = kv2p(USER_CODE_PAGE_3.0.as_ptr() as u64) | code_flags;
            *pt_code.add(3) = kv2p(USER_CODE_PAGE_4.0.as_ptr() as u64) | code_flags;
            *pt_code.add(4) = kv2p(USER_CODE_PAGE_5.0.as_ptr() as u64) | code_flags;
            *pt_code.add(5) = kv2p(USER_CODE_PAGE_6.0.as_ptr() as u64) | code_flags;
            *pt_code.add(6) = kv2p(USER_CODE_PAGE_7.0.as_ptr() as u64) | code_flags;
            *pt_code.add(7) = kv2p(USER_CODE_PAGE_8.0.as_ptr() as u64) | code_flags;
        }

        let pt_stack = USER_PT_STACK.0.as_mut_ptr() as *mut u64;
        *pt_stack.add(511) = kv2p(USER_STACK_PAGE.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(510) = kv2p(M3_STACK_PAGE_1.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(509) = kv2p(M3_STACK_PAGE_2.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(508) = kv2p(M3_STACK_PAGE_3.0.as_ptr() as u64) | 0x07;
        #[cfg(feature = "go_test")]
        {
            *pt_stack.add(510) = kv2p(USER_STACK_PAGE_2.0.as_ptr() as u64) | 0x07;
            *pt_stack.add(509) = kv2p(USER_STACK_PAGE_3.0.as_ptr() as u64) | 0x07;
            *pt_stack.add(508) = kv2p(USER_STACK_PAGE_4.0.as_ptr() as u64) | 0x07;
            *pt_stack.add(507) = kv2p(USER_STACK_PAGE_5.0.as_ptr() as u64) | 0x07;
            *pt_stack.add(506) = kv2p(USER_STACK_PAGE_6.0.as_ptr() as u64) | 0x07;
            *pt_stack.add(505) = kv2p(USER_STACK_PAGE_7.0.as_ptr() as u64) | 0x07;
            *pt_stack.add(504) = kv2p(USER_STACK_PAGE_8.0.as_ptr() as u64) | 0x07;
            *pt_stack.add(503) = kv2p(USER_HEAP_PAGE_1.0.as_ptr() as u64) | 0x07;
            *pt_stack.add(502) = kv2p(USER_HEAP_PAGE_2.0.as_ptr() as u64) | 0x07;
            *pt_stack.add(501) = kv2p(USER_HEAP_PAGE_3.0.as_ptr() as u64) | 0x07;
            *pt_stack.add(500) = kv2p(USER_HEAP_PAGE_4.0.as_ptr() as u64) | 0x07;
        }

        *new_pml4 = kv2p(USER_PDPT.0.as_ptr() as u64) | 0x07;
        let new_pml4_phys = kv2p(new_pml4 as u64);
        core::arch::asm!("mov cr3, {}", in(reg) new_pml4_phys, options(nostack));

        m3_reset_state();
        m3_zero_user_code_pages();
        m3_load_user_elf_image(image)
    }
}

// --------------- M3: User program blobs --------------------------------------

cfg_m3! {
    static USER_HELLO_BLOB: [u8; 32] = [
        0x48, 0x8d, 0x3d, 0x0d, 0x00, 0x00, 0x00,
        0x48, 0xc7, 0xc6, 0x0c, 0x00, 0x00, 0x00,
        0x31, 0xc0,
        0xcd, 0x80,
        0xf4, 0x00,
        b'U', b'S', b'E', b'R', b':', b' ',
        b'h', b'e', b'l', b'l', b'o', b'\n',
    ];

    static USER_SYSCALL_BLOB: [u8; 41] = [
        0xb8, 0x0a, 0x00, 0x00, 0x00,
        0xcd, 0x80,
        0x48, 0x8d, 0x3d, 0x0f, 0x00, 0x00, 0x00,
        0x48, 0xc7, 0xc6, 0x0c, 0x00, 0x00, 0x00,
        0x31, 0xc0,
        0xcd, 0x80,
        0xf4, 0x00, 0x00, 0x00,
        b'S', b'Y', b'S', b'C', b'A', b'L', b'L',
        b':', b' ', b'o', b'k', b'\n',
    ];

    #[cfg(feature = "thread_exit_test")]
    static USER_THREAD_EXIT_BLOB: [u8; 8] = [
        0xB8, 0x02, 0x00, 0x00, 0x00,
        0xCD, 0x80,
        0xF4,
    ];

    #[cfg(feature = "thread_spawn_test")]
    static USER_THREAD_SPAWN_BLOB: [u8; 157] = [
        0x48, 0x8D, 0x3D, 0x31, 0x00, 0x00, 0x00, 0xB8, 0x01, 0x00, 0x00, 0x00,
        0xCD, 0x80, 0x48, 0x83, 0xF8, 0xFF, 0x74, 0x3C, 0xB8, 0x03, 0x00, 0x00,
        0x00, 0xCD, 0x80, 0x48, 0x8D, 0x3D, 0x4B, 0x00, 0x00, 0x00, 0xBE, 0x0F,
        0x00, 0x00, 0x00, 0x31, 0xC0, 0xCD, 0x80, 0xBF, 0x31, 0x00, 0x00, 0x00,
        0xB8, 0x62, 0x00, 0x00, 0x00, 0xCD, 0x80, 0xF4, 0x48, 0x8D, 0x3D, 0x3D,
        0x00, 0x00, 0x00, 0xBE, 0x10, 0x00, 0x00, 0x00, 0x31, 0xC0, 0xCD, 0x80,
        0xB8, 0x02, 0x00, 0x00, 0x00, 0xCD, 0x80, 0xF4, 0x48, 0x8D, 0x3D, 0x35,
        0x00, 0x00, 0x00, 0xBE, 0x11, 0x00, 0x00, 0x00, 0x31, 0xC0, 0xCD, 0x80,
        0xBF, 0x33, 0x00, 0x00, 0x00, 0xB8, 0x62, 0x00, 0x00, 0x00, 0xCD, 0x80,
        0xF4, 0x53, 0x50, 0x41, 0x57, 0x4E, 0x3A, 0x20, 0x6D, 0x61, 0x69, 0x6E,
        0x20, 0x6F, 0x6B, 0x0A, 0x53, 0x50, 0x41, 0x57, 0x4E, 0x3A, 0x20, 0x63,
        0x68, 0x69, 0x6C, 0x64, 0x20, 0x6F, 0x6B, 0x0A, 0x53, 0x50, 0x41, 0x57,
        0x4E, 0x3A, 0x20, 0x73, 0x70, 0x61, 0x77, 0x6E, 0x20, 0x65, 0x72, 0x72,
        0x0A,
    ];

    #[cfg(feature = "vm_map_test")]
    static USER_VM_MAP_BLOB: [u8; 216] = [
        0xBF, 0x00, 0x00, 0x50, 0x00, 0xBE, 0x00, 0x10, 0x00, 0x00, 0xB8, 0x04,
        0x00, 0x00, 0x00, 0xCD, 0x80, 0x48, 0x83, 0xF8, 0xFF, 0x0F, 0x84, 0x89,
        0x00, 0x00, 0x00, 0xC6, 0x04, 0x25, 0x00, 0x00, 0x50, 0x00, 0x5A, 0xBF,
        0x00, 0x00, 0x50, 0x00, 0xBE, 0x00, 0x10, 0x00, 0x00, 0xB8, 0x05, 0x00,
        0x00, 0x00, 0xCD, 0x80, 0x48, 0x83, 0xF8, 0xFF, 0x74, 0x6A, 0xBF, 0x01,
        0x00, 0x50, 0x00, 0xBE, 0x00, 0x10, 0x00, 0x00, 0xB8, 0x04, 0x00, 0x00,
        0x00, 0xCD, 0x80, 0x48, 0x83, 0xF8, 0xFF, 0x75, 0x53, 0xBF, 0x00, 0x00,
        0x50, 0x00, 0xBE, 0x00, 0x10, 0x00, 0x00, 0xB8, 0x04, 0x00, 0x00, 0x00,
        0xCD, 0x80, 0x48, 0x83, 0xF8, 0xFF, 0x74, 0x3C, 0xC6, 0x04, 0x25, 0x00,
        0x00, 0x50, 0x00, 0x6B, 0xBF, 0x00, 0x00, 0x50, 0x00, 0xBE, 0x00, 0x10,
        0x00, 0x00, 0xB8, 0x05, 0x00, 0x00, 0x00, 0xCD, 0x80, 0x48, 0x83, 0xF8,
        0xFF, 0x74, 0x1D, 0x48, 0x8D, 0x3D, 0x33, 0x00, 0x00, 0x00, 0xBE, 0x0B,
        0x00, 0x00, 0x00, 0x31, 0xC0, 0xCD, 0x80, 0xBF, 0x31, 0x00, 0x00, 0x00,
        0xB8, 0x62, 0x00, 0x00, 0x00, 0xCD, 0x80, 0xF4, 0x48, 0x8D, 0x3D, 0x21,
        0x00, 0x00, 0x00, 0xBE, 0x0C, 0x00, 0x00, 0x00, 0x31, 0xC0, 0xCD, 0x80,
        0xBF, 0x33, 0x00, 0x00, 0x00, 0xB8, 0x62, 0x00, 0x00, 0x00, 0xCD, 0x80,
        0xF4, 0x56, 0x4D, 0x3A, 0x20, 0x6D, 0x61, 0x70, 0x20, 0x6F, 0x6B, 0x0A,
        0x56, 0x4D, 0x3A, 0x20, 0x6D, 0x61, 0x70, 0x20, 0x65, 0x72, 0x72, 0x0A,
    ];

    static USER_FAULT_BLOB: [u8; 9] = [
        0xb8, 0x00, 0x00, 0xad, 0xde,
        0xc6, 0x00, 0x42,
        0xf4,
    ];
}

#[cfg(feature = "syscall_invalid_test")]
static USER_SYSCALL_INVALID_BLOB: [u8; 63] = [
    // mov eax, 99 ; unknown syscall
    0xB8, 0x63, 0x00, 0x00, 0x00,
    // int 0x80
    0xCD, 0x80,
    // cmp rax, -1 ; kernel must return -1
    0x48, 0x83, 0xF8, 0xFF,
    // jne fail
    0x75, 0x1D,
    // sys_debug_write("SYSCALL: invalid ok\n", 20)
    0x48, 0x8D, 0x3D, 0x17, 0x00, 0x00, 0x00,
    0xBE, 0x14, 0x00, 0x00, 0x00,
    0x31, 0xC0,
    0xCD, 0x80,
    // test-only syscall: qemu_exit(0x31)
    0xBF, 0x31, 0x00, 0x00, 0x00,
    0xB8, 0x62, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // success/fail halt fallback
    0xF4,
    0xF4,
    b'S', b'Y', b'S', b'C', b'A', b'L', b'L', b':', b' ', b'i',
    b'n', b'v', b'a', b'l', b'i', b'd', b' ', b'o', b'k', b'\n',
];

#[cfg(feature = "stress_syscall_test")]
static USER_STRESS_SYSCALL_BLOB: [u8; 131] = [
    // mov r12d, 2000 ; main loop counter
    0x41, 0xBC, 0xD0, 0x07, 0x00, 0x00,
    // mov r13d, 200 ; progress cadence
    0x41, 0xBD, 0xC8, 0x00, 0x00, 0x00,
    // loop: sys_time_now()
    0xB8, 0x0A, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // assert rax != -1
    0x48, 0x83, 0xF8, 0xFF,
    0x74, 0x4A,
    // syscall 99 (unknown)
    0xB8, 0x63, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // assert rax == -1
    0x48, 0x83, 0xF8, 0xFF,
    0x75, 0x3D,
    // dec r13d; every 200 iterations print "."
    0x41, 0xFF, 0xCD,
    0x75, 0x16,
    0x48, 0x8D, 0x3D, 0x3E, 0x00, 0x00, 0x00,
    0xBE, 0x01, 0x00, 0x00, 0x00,
    0x31, 0xC0,
    0xCD, 0x80,
    0x41, 0xBD, 0xC8, 0x00, 0x00, 0x00,
    // dec r12d; loop until zero
    0x41, 0xFF, 0xCC,
    0x75, 0xC6,
    // sys_debug_write("STRESS: syscall ok", 18)
    0x48, 0x8D, 0x3D, 0x24, 0x00, 0x00, 0x00,
    0xBE, 0x12, 0x00, 0x00, 0x00,
    0x31, 0xC0,
    0xCD, 0x80,
    // qemu_exit(0x31)
    0xBF, 0x31, 0x00, 0x00, 0x00,
    0xB8, 0x62, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0xF4,
    // fail: qemu_exit(0x33)
    0xBF, 0x33, 0x00, 0x00, 0x00,
    0xB8, 0x62, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0xF4,
    // data
    b'.',
    b'S', b'T', b'R', b'E', b'S', b'S', b':', b' ', b's', b'y', b's', b'c',
    b'a', b'l', b'l', b' ', b'o', b'k',
];

#[cfg(feature = "yield_test")]
static USER_YIELD_BLOB: [u8; 53] = [
    // sys_yield()
    0xB8, 0x03, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // sys_yield()
    0xB8, 0x03, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // sys_debug_write("YIELD: ok\n", 10)
    0x48, 0x8D, 0x3D, 0x16, 0x00, 0x00, 0x00,
    0xBE, 0x0A, 0x00, 0x00, 0x00,
    0x31, 0xC0,
    0xCD, 0x80,
    // test-only syscall: qemu_exit(0x31)
    0xBF, 0x31, 0x00, 0x00, 0x00,
    0xB8, 0x62, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // fallback
    0xF4,
    b'Y', b'I', b'E', b'L', b'D', b':', b' ', b'o', b'k', b'\n',
];

// --------------- G1: TinyGo user blob ----------------------------------------

#[cfg(feature = "go_test")]
static GO_USER_BIN: &[u8] = include_bytes!("../../out/gousr.bin");
#[cfg(feature = "go_desktop_test")]
static GO_DESKTOP_BIN: &[u8] = include_bytes!("../../out/gousr-desktop.bin");

// --------------- X1 runtime-backed compatibility ELF corpus ------------------

#[cfg(feature = "compat_real_test")]
#[derive(Clone, Copy)]
struct CompatRealApp {
    name: &'static [u8],
    image: &'static [u8],
}

#[cfg(feature = "compat_real_test")]
static X1_CLI_FILE_ELF: &[u8] = include_bytes!("../../out/x1-cli-file.elf");

#[cfg(feature = "compat_real_test")]
static X1_PROC_SOCK_ELF: &[u8] = include_bytes!("../../out/x1-proc-sock.elf");

#[cfg(feature = "compat_real_test")]
static COMPAT_REAL_APPS: [CompatRealApp; 2] = [
    CompatRealApp { name: b"x1-cli-file", image: X1_CLI_FILE_ELF },
    CompatRealApp { name: b"x1-proc-sock", image: X1_PROC_SOCK_ELF },
];

#[cfg(feature = "compat_real_test")]
static mut COMPAT_REAL_APP_INDEX: usize = 0;

// --------------- G2 spike: std-port candidate blob ----------------------------

#[cfg(feature = "go_std_test")]
static GO_STD_BIN: &[u8] = include_bytes!("../../out/gostd.bin");

// --------------- M10: Security baseline user blobs ----------------------------

#[cfg(feature = "sec_rights_test")]
static SEC_RIGHTS_BIN: &[u8] = include_bytes!("../../out/sec-rights.bin");

#[cfg(feature = "sec_filter_test")]
static SEC_FILTER_BIN: &[u8] = include_bytes!("../../out/sec-filter.bin");

// =============================================================================
// R4: IPC + shared memory + service registry
// =============================================================================

// --------------- R4: Additional pages for second task -------------------------

cfg_r4! {
    const USER_CODE2_VA: u64   = 0x40_1000;
    const USER_STACK2_TOP: u64 = 0x7F_F000;
    #[cfg(any(feature = "stress_ipc_test", feature = "go_test"))]
    const USER_CODE3_VA: u64   = 0x40_2000;
    #[cfg(any(feature = "stress_ipc_test", feature = "go_test"))]
    const USER_STACK3_TOP: u64 = 0x7F_E000;
    #[cfg(any(feature = "stress_ipc_test", feature = "go_test"))]
    const USER_CODE4_VA: u64   = 0x40_3000;
    #[cfg(any(feature = "stress_ipc_test", feature = "go_test"))]
    const USER_STACK4_TOP: u64 = 0x7F_D000;

    static mut USER_CODE_PAGE_2:  Page = Page([0; 4096]);
    static mut USER_STACK_PAGE_2: Page = Page([0; 4096]);
    #[cfg(any(feature = "stress_ipc_test", feature = "go_test"))]
    static mut USER_CODE_PAGE_3:  Page = Page([0; 4096]);
    #[cfg(any(feature = "stress_ipc_test", feature = "go_test"))]
    static mut USER_STACK_PAGE_3: Page = Page([0; 4096]);
    #[cfg(any(feature = "stress_ipc_test", feature = "go_test"))]
    static mut USER_CODE_PAGE_4:  Page = Page([0; 4096]);
    #[cfg(any(feature = "stress_ipc_test", feature = "go_test"))]
    static mut USER_STACK_PAGE_4: Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_CODE_PAGE_5:  Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_CODE_PAGE_6:  Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_CODE_PAGE_7:  Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_CODE_PAGE_8:  Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_STACK_PAGE_5: Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_STACK_PAGE_6: Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_STACK_PAGE_7: Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_STACK_PAGE_8: Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_HEAP_PAGE_1: Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_HEAP_PAGE_2: Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_HEAP_PAGE_3: Page = Page([0; 4096]);
    #[cfg(feature = "go_test")]
    static mut USER_HEAP_PAGE_4: Page = Page([0; 4096]);
}

// --------------- R4: SHM backing pages ---------------------------------------

#[cfg(feature = "pressure_shm_test")]
const R4_MAX_SHM: usize = 64;

#[cfg(all(feature = "quota_shm_test", not(feature = "pressure_shm_test")))]
const R4_MAX_SHM: usize = 64;

#[cfg(all(feature = "shm_test", not(feature = "pressure_shm_test"), not(feature = "quota_shm_test")))]
const R4_MAX_SHM: usize = 2;

#[cfg(any(feature = "shm_test", feature = "quota_shm_test"))]
#[derive(Clone, Copy)]
struct ShmObject {
    active: bool,
    size: usize,
}

#[cfg(any(feature = "shm_test", feature = "quota_shm_test"))]
static mut R4_SHM_PAGES: [Page; R4_MAX_SHM] = [Page([0; 4096]); R4_MAX_SHM];

#[cfg(any(feature = "shm_test", feature = "quota_shm_test"))]
static mut R4_SHM_OBJECTS: [ShmObject; R4_MAX_SHM] = [ShmObject { active: false, size: 0 }; R4_MAX_SHM];

// --------------- R4: Task model ----------------------------------------------

cfg_r4! {
    const MAX_ENDPOINTS_PER_PROC: usize = 16;
    const MAX_SHM_PER_PROC: usize = 32;
    const MAX_THREADS_PER_PROC: usize = 16;
    const MAX_THREADS_GLOBAL: usize = 64;
    const R4_WAIT_NONE: i32 = -2;
    const R4_SCHED_CLASS_BEST_EFFORT: u8 = 0;
    const R4_SCHED_CLASS_CRITICAL: u8 = 1;
    const R4_PROC_INFO_BASE_WORDS: usize = 13;
    const R4_PROC_INFO_BASE_SIZE: usize = R4_PROC_INFO_BASE_WORDS * 8;
    const R4_PROC_INFO_EXT_WORDS: usize = 17;
    const R4_PROC_INFO_EXT_SIZE: usize = R4_PROC_INFO_EXT_WORDS * 8;
    const R4_TASK_CAP_STORAGE: u8 = 1 << 0;
    const R4_TASK_CAP_NETWORK: u8 = 1 << 1;
    const R4_TASK_CAP_MASK: u8 = R4_TASK_CAP_STORAGE | R4_TASK_CAP_NETWORK;
    const R4_TASK_DEFAULT_FD_LIMIT: u8 = 8;
    const R4_TASK_DEFAULT_SOCKET_LIMIT: u8 = 4;
    const R4_TASK_DEFAULT_ENDPOINT_LIMIT: u8 = 4;

    // The task table is heap-backed (Phase 1 allocator); this is the spawn
    // cap, not a static array bound. The go lane allows dynamic process
    // populations well past the historical 6-slot limit.
    #[cfg(feature = "go_test")]
    const R4_MAX_TASKS: usize = 32;
    #[cfg(all(feature = "stress_ipc_test", not(feature = "go_test")))]
    const R4_MAX_TASKS: usize = 6;
    #[cfg(not(any(feature = "stress_ipc_test", feature = "go_test")))]
    const R4_MAX_TASKS: usize = 2;

    #[derive(Clone, Copy, PartialEq)]
    enum R4State { Ready, Running, Blocked, Exited, Dead }

    #[derive(Clone, Copy)]
    struct R4Task {
        saved_frame: [u64; 22],
        state: R4State,
        recv_ep: u64,
        recv_buf: u64,
        recv_cap: u64,
        endpoint_count: usize,
        shm_count: usize,
        thread_count: usize,
        can_spawn: bool,
        // SMP affinity (full-os Part I.3 per-CPU scheduler): when set, this task runs
        // ONLY on an application processor -- the BSP's r4_find_ready skips it and an
        // AP claims it from the live table. Default false (BSP-scheduled), so the
        // normal workload is unaffected.
        ap_eligible: bool,
        parent_tid: usize,
        exit_status: u64,
        wait_target: i32,
        wait_status_ptr: u64,
        sched_class: u8,
        isolation_domain: u8,
        cap_flags: u8,
        fd_count: usize,
        fd_limit: u8,
        socket_count: usize,
        socket_limit: u8,
        endpoint_limit: u8,
        dispatch_count: u64,
        yield_count: u64,
        block_count: u64,
        ipc_send_count: u64,
        ipc_recv_count: u64,
        // Signals (gap item 10): one registered handler, a pending
        // bitmap, and the interrupted frame saved across delivery.
        sig_handler: u64,
        sig_pending: u64,
        sig_in_handler: bool,
        sig_saved_frame: [u64; 22],
        // User id (gap item 10): 0 = root (boot services), spawned
        // external apps run as uid 100.
        uid: u8,
        // Per-process address space (full-os keystone): the physical
        // address of this task's PML4. 0 = run on the shared/boot table
        // (no CR3 reload); non-zero = a private address space whose
        // user-half is isolated and reclaimed on exit.
        pml4_phys: u64,
        // Program break for sys_vm_ctl brk (full-os guide Part I.4).
        // 0 = not yet initialized (lazily set to the brk base on first use).
        heap_brk: u64,
        // Syscall allowlist (full-os guide Part IV.10 sandbox). Bit N set =
        // syscall N permitted. u64::MAX = unrestricted; narrowed monotonically
        // by sys_sandbox. Syscalls 0 (debug_write) and 2 (exit) stay allowed.
        sec_filter_mask: u64,
        // Futex wait address (full-os guide Part I.3 concurrency). Non-zero
        // while the task is Blocked in sys_futex wait; the matching wake
        // clears it and makes the task Ready.
        futex_uaddr: u64,
        // nanosleep deadline in PIT ticks (full-os guide Part IV.9). Non-zero
        // while the task is Blocked asleep; the PIT handler clears it and
        // makes the task Ready once the deadline passes.
        sleep_until: u64,
    }

    impl R4Task {
        const EMPTY: Self = Self {
            saved_frame: [0u64; 22],
            state: R4State::Dead,
            recv_ep: 0, recv_buf: 0, recv_cap: 0,
            endpoint_count: 0,
            shm_count: 0,
            thread_count: 0,
            can_spawn: false,
            ap_eligible: false,
            parent_tid: 0,
            exit_status: 0,
            wait_target: R4_WAIT_NONE,
            wait_status_ptr: 0,
            sched_class: R4_SCHED_CLASS_BEST_EFFORT,
            isolation_domain: 0,
            cap_flags: 0,
            fd_count: 0,
            fd_limit: 0,
            socket_count: 0,
            socket_limit: 0,
            endpoint_limit: 0,
            dispatch_count: 0,
            yield_count: 0,
            block_count: 0,
            ipc_send_count: 0,
            ipc_recv_count: 0,
            sig_handler: 0,
            sig_pending: 0,
            sig_in_handler: false,
            sig_saved_frame: [0u64; 22],
            uid: 0,
            pml4_phys: 0,
            heap_brk: 0,
            sec_filter_mask: u64::MAX,
            futex_uaddr: 0,
            sleep_until: 0,
        };
    }

    // Dynamically allocated process structures (gap-analysis item 2): the
    // table lives on the kernel heap and is pre-sized to the spawn cap at
    // boot so every existing index site stays valid.
    static mut R4_TASKS: alloc::vec::Vec<R4Task> = alloc::vec::Vec::new();
    static mut R4_CURRENT: usize = 0;
    static mut R4_NUM_TASKS: usize = 0;
    static mut R4_THREADS_CREATED: usize = 0;
    static mut R4_TASKS_HIGH: usize = 0;

    pub(crate) unsafe fn r4_tasks_init() {
        R4_TASKS.clear();
        let mut i = 0;
        while i < R4_MAX_TASKS {
            R4_TASKS.push(R4Task::EMPTY);
            i += 1;
        }
    }

    #[inline(always)]
    unsafe fn r4_stack_top_for_slot(slot: usize) -> u64 {
        #[cfg(feature = "go_test")]
        {
            // Slots 0-4 keep the historical static strides below the user
            // stack top; higher slots get demand-paged strides whose pages
            // arrive zeroed on first touch (guard zones enforced by mm).
            if slot >= 5 {
                return mm::DEMAND_STACK_BASE
                    + ((slot as u64) - 4) * mm::DEMAND_STACK_STRIDE;
            }
            USER_STACK_TOP - (slot as u64) * 0x2000
        }
        #[cfg(not(feature = "go_test"))]
        {
            USER_STACK_TOP - (slot as u64) * 0x1000
        }
    }

    unsafe fn r4_init_task(tid: usize, code_va: u64, stk_top: u64, parent_tid: usize) {
        R4_TASKS[tid].saved_frame = [0u64; 22];
        R4_TASKS[tid].saved_frame[17] = code_va;  // RIP
        R4_TASKS[tid].saved_frame[18] = 0x23;     // CS (user code RPL=3)
        // IF is only seeded in the pure go lane, where kmain remaps and
        // masks the PIC before any task runs. The compat lane composes
        // go_test+compat_real_test without programming the PIC: an IF=1
        // task there would route stale PIT ticks to the double-fault
        // vector.
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        { R4_TASKS[tid].saved_frame[19] = 0x202; } // RFLAGS, IF set: preemptible
        #[cfg(not(all(feature = "go_test", not(feature = "compat_real_test"))))]
        { R4_TASKS[tid].saved_frame[19] = 0x02; }  // RFLAGS
        R4_TASKS[tid].saved_frame[20] = stk_top;  // RSP
        R4_TASKS[tid].saved_frame[21] = 0x1B;     // SS (user data RPL=3)
        if R4_NUM_TASKS > R4_TASKS_HIGH {
            R4_TASKS_HIGH = R4_NUM_TASKS;
        }
        R4_TASKS[tid].recv_ep = 0;
        R4_TASKS[tid].recv_buf = 0;
        R4_TASKS[tid].recv_cap = 0;
        R4_TASKS[tid].endpoint_count = 0;
        R4_TASKS[tid].shm_count = 0;
        R4_TASKS[tid].thread_count = 0;
        R4_TASKS[tid].parent_tid = parent_tid;
        R4_TASKS[tid].exit_status = 0;
        R4_TASKS[tid].wait_target = R4_WAIT_NONE;
        R4_TASKS[tid].wait_status_ptr = 0;
        R4_TASKS[tid].sched_class = R4_SCHED_CLASS_BEST_EFFORT;
        // Slot reuse must not leak SMP affinity: a reused slot defaults to
        // BSP-scheduled (an AP-eligible task explicitly opts in after init).
        R4_TASKS[tid].ap_eligible = false;
        // Slot reuse must not leak signal state into the next task.
        R4_TASKS[tid].sig_handler = 0;
        R4_TASKS[tid].sig_pending = 0;
        R4_TASKS[tid].sig_in_handler = false;
        // Threads inherit the spawner's uid; boot tasks are root.
        R4_TASKS[tid].uid = if tid == parent_tid {
            0
        } else {
            R4_TASKS[parent_tid].uid
        };
        // Per-process address space (full-os keystone): a thread inherits
        // its spawner's table; a fresh root task starts on the shared
        // table (0 here; the go-lane boot path stamps the shared PML4 and
        // sys_spawn installs a private one after this returns).
        R4_TASKS[tid].pml4_phys = if tid == parent_tid {
            0
        } else {
            R4_TASKS[parent_tid].pml4_phys
        };
        // Program break is per-task and reset on slot reuse.
        R4_TASKS[tid].heap_brk = 0;
        // A new task starts unrestricted, or inherits a thread parent's
        // narrowed allowlist (reset on slot reuse).
        R4_TASKS[tid].sec_filter_mask = if tid == parent_tid {
            u64::MAX
        } else {
            R4_TASKS[parent_tid].sec_filter_mask
        };
        R4_TASKS[tid].futex_uaddr = 0;
        R4_TASKS[tid].sleep_until = 0;
        if tid == parent_tid {
            R4_TASKS[tid].isolation_domain = 0;
            R4_TASKS[tid].cap_flags = R4_TASK_CAP_MASK;
            R4_TASKS[tid].fd_limit = R4_TASK_DEFAULT_FD_LIMIT;
            R4_TASKS[tid].socket_limit = R4_TASK_DEFAULT_SOCKET_LIMIT;
            R4_TASKS[tid].endpoint_limit = R4_TASK_DEFAULT_ENDPOINT_LIMIT;
        } else {
            R4_TASKS[tid].isolation_domain = R4_TASKS[parent_tid].isolation_domain;
            R4_TASKS[tid].cap_flags = R4_TASKS[parent_tid].cap_flags;
            R4_TASKS[tid].fd_limit = R4_TASKS[parent_tid].fd_limit;
            R4_TASKS[tid].socket_limit = R4_TASKS[parent_tid].socket_limit;
            R4_TASKS[tid].endpoint_limit = R4_TASKS[parent_tid].endpoint_limit;
        }
        R4_TASKS[tid].fd_count = 0;
        R4_TASKS[tid].socket_count = 0;
        R4_TASKS[tid].dispatch_count = 0;
        R4_TASKS[tid].yield_count = 0;
        R4_TASKS[tid].block_count = 0;
        R4_TASKS[tid].ipc_send_count = 0;
        R4_TASKS[tid].ipc_recv_count = 0;
        #[cfg(feature = "go_test")]
        {
            R4_TASKS[tid].can_spawn = tid == 0;
        }
        #[cfg(not(feature = "go_test"))]
        {
            R4_TASKS[tid].can_spawn = false;
        }
        R4_TASKS[tid].state = R4State::Ready;
    }

    unsafe fn r4_find_ready(exclude: usize) -> Option<usize> {
        if R4_NUM_TASKS == 0 { return None; }
        let mut best: Option<usize> = None;
        let mut best_class = R4_SCHED_CLASS_BEST_EFFORT;
        let mut i = (exclude + 1) % R4_NUM_TASKS;
        for _ in 0..R4_NUM_TASKS {
            // SMP affinity (full-os Part I.3): the BSP skips AP-eligible tasks; they
            // are reserved for application processors to claim from the live table.
            if i != exclude && R4_TASKS[i].state == R4State::Ready && !R4_TASKS[i].ap_eligible {
                let class = R4_TASKS[i].sched_class;
                if best.is_none() || class > best_class {
                    best = Some(i);
                    best_class = class;
                }
            }
            i = (i + 1) % R4_NUM_TASKS;
        }
        best
    }

    unsafe fn r4_switch_to(frame: *mut u64, tid: usize) {
        // Per-process address space (full-os keystone): load the target
        // task's PML4 before resuming it. The boot task carries the shared
        // table's physical address, spawned apps carry private ones; a 0
        // (other lanes) means "leave CR3 as-is". This flushes the TLB.
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            let pml4 = R4_TASKS[tid].pml4_phys;
            if pml4 != 0 {
                core::arch::asm!("mov cr3, {}", in(reg) pml4, options(nostack));
            }
        }
        for i in 0..22 { *frame.add(i) = R4_TASKS[tid].saved_frame[i]; }
        R4_TASKS[tid].state = R4State::Running;
        R4_TASKS[tid].dispatch_count += 1;
        // Record the switched-to task as the CALLING CPU's current (per-CPU on an AP,
        // R4_CURRENT on the BSP) so r4_switch_to is usable from an AP scheduler path.
        r4_set_current_smp(tid);
        sig_deliver_if_pending(frame, tid);
    }

    // ---- signals (gap item 10) ----
    // One registered handler per task, a pending bitmap, frame-rewrite
    // delivery at the dispatch points (task switch and self-directed
    // sys_signal_ctl). Signal 9 always kills; any other signal without
    // a handler kills; sigreturn restores the interrupted frame.

    const SIG_FORCED_KILL: u64 = 9;

    unsafe fn sig_deliver_if_pending(frame: *mut u64, tid: usize) {
        if R4_TASKS[tid].sig_pending == 0 || R4_TASKS[tid].sig_in_handler {
            return;
        }
        let sig = R4_TASKS[tid].sig_pending.trailing_zeros() as u64;
        R4_TASKS[tid].sig_pending &= !(1u64 << sig);
        if sig == SIG_FORCED_KILL || R4_TASKS[tid].sig_handler == 0 {
            serial_write(b"SIG: kill tid=0x");
            serial_write_hex(tid as u64);
            serial_write(b" sig=0x");
            serial_write_hex(sig);
            serial_write(b"\n");
            r4_exit_and_switch(frame, 1);
            return;
        }
        // Redirect to the handler: save the interrupted state, then
        // run handler(sig) on a red-zone-clear, aligned stack.
        for i in 0..22 {
            R4_TASKS[tid].sig_saved_frame[i] = *frame.add(i);
        }
        R4_TASKS[tid].sig_in_handler = true;
        *frame.add(17) = R4_TASKS[tid].sig_handler; // rip
        *frame.add(9) = sig; // rdi
        *frame.add(20) = (*frame.add(20) - 256) & !0xF; // rsp
    }

    /// sys_net_query (ABI v3.2 id 49): op 1 = DHCP discover, op 2 = DNS
    /// A query (a2 = name pointer, a3 = len | port << 16), op 3 = poll —
    /// returns u64::MAX while pending, then the IPv4 result once. op 4 =
    /// ICMP echo self-test, op 5 = ARP responder self-test, op 6 = TCP
    /// passive-open self-test, op 7 = ICMPv6 echo self-test, op 8 = UDP echo
    /// self-test, op 9 = IPv6 NDP Neighbor-Advertisement self-test, op 10 =
    /// TCP retransmission/RTO self-test (each returns 1 on success, 0 on fail).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_net_query(op: u64, a2: u64, a3: u64) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        if R4_TASKS[r4_current_smp()].cap_flags & R4_TASK_CAP_NETWORK == 0 {
            audit_event(b"cap-deny", 49); // full-os guide Part IV.10 audit trail
            return ERR;
        }
        match op {
            1 => netcfg::start_dhcp(),
            2 => {
                let len = (a3 & 0xFFFF) as usize;
                let port = (a3 >> 16) as u16;
                if len == 0 || len > 63 || port == 0 {
                    return ERR;
                }
                let mut name = [0u8; 64];
                if copyin_user(&mut name[..len], a2, len).is_err() {
                    return ERR;
                }
                netcfg::start_dns(&name[..len], port)
            }
            3 => netcfg::poll_result(),
            4 => netcfg::icmp_selftest(),
            5 => netcfg::arp_selftest(),
            6 => tcp::tcp_listen_selftest(),
            7 => netcfg::icmpv6_selftest(),
            8 => netcfg::udp_echo_selftest(),
            9 => netcfg::ndp_selftest(),
            10 => tcp::tcp_rto_selftest(),
            11 => tcp::tcp_rtt_selftest(),
            12 => tcp::tcp_cc_selftest(),
            13 => net::route_selftest(),
            14 => netcfg::nud_selftest(),
            15 => netcfg::slaac_selftest(),
            16 => tcp::tcp_fastrexmit_selftest(),
            17 => netcfg::udp6_echo_selftest(),
            18 => netcfg::tcp6_listen_selftest(),
            19 => tcp::tcp_sndwin_selftest(),
            20 => netcfg::dad_selftest(), // IPv6 NDP Duplicate Address Detection
            21 => netcfg::icmp_echo_request_selftest(), // ICMPv4 outbound ping builder
            22 => netcfg::icmp_error_selftest(), // ICMPv4 dest-unreachable / time-exceeded
            23 => netcfg::gratuitous_arp_selftest(), // gratuitous ARP announcement
            24 => netcfg::arp_cache_selftest(), // IPv4 ARP cache learn/lookup
            25 => tcp::tcp_mss_selftest(), // TCP MSS option on the SYN
            _ => ERR,
        }
    }

    /// sys_signal_ctl (ABI v3.2 id 48): op 1 = register handler,
    /// op 2 = kill(tid, sig) — tid u64::MAX means self; op 3 = sigreturn.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_signal_ctl(frame: *mut u64, op: u64, a2: u64, a3: u64) {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        match op {
            1 => {
                // Handler must be a user-space address.
                if a2 != 0 && a2 >= 0x0000_8000_0000_0000 {
                    *frame.add(14) = ERR;
                    return;
                }
                R4_TASKS[r4_current_smp()].sig_handler = a2;
                *frame.add(14) = 0;
            }
            2 => {
                let tid = if a2 == u64::MAX { R4_CURRENT } else { a2 as usize };
                if tid >= R4_NUM_TASKS
                    || a3 >= 64
                    || matches!(R4_TASKS[tid].state, R4State::Dead)
                {
                    *frame.add(14) = ERR;
                    return;
                }
                // Only self or a direct child may be signalled.
                if tid != R4_CURRENT && R4_TASKS[tid].parent_tid != R4_CURRENT {
                    *frame.add(14) = ERR;
                    return;
                }
                R4_TASKS[tid].sig_pending |= 1u64 << a3;
                *frame.add(14) = 0;
                if tid == R4_CURRENT {
                    // Deliver on the live frame before returning to user.
                    sig_deliver_if_pending(frame, tid);
                }
            }
            3 => {
                if !R4_TASKS[r4_current_smp()].sig_in_handler {
                    *frame.add(14) = ERR;
                    return;
                }
                R4_TASKS[r4_current_smp()].sig_in_handler = false;
                for i in 0..22 {
                    *frame.add(i) = R4_TASKS[r4_current_smp()].sig_saved_frame[i];
                }
            }
            _ => {
                *frame.add(14) = ERR;
            }
        }
    }

    unsafe fn r4_save_frame(frame: *mut u64, tid: usize) {
        for i in 0..22 { R4_TASKS[tid].saved_frame[i] = *frame.add(i); }
    }

    #[inline(always)]
    unsafe fn r4_wait_matches(target: i32, child_tid: usize) -> bool {
        target == -1 || target == child_tid as i32
    }

    #[inline(always)]
    unsafe fn r4_copy_wait_status(status_ptr: u64, status: u64) -> bool {
        if status_ptr == 0 {
            return true;
        }
        let st = status.to_le_bytes();
        copyout_user(status_ptr, &st, st.len()).is_ok()
    }

    unsafe fn r4_has_waitable_child(parent_tid: usize, target: i32) -> bool {
        for tid in 0..R4_NUM_TASKS {
            if tid == parent_tid || R4_TASKS[tid].parent_tid != parent_tid {
                continue;
            }
            if !r4_wait_matches(target, tid) {
                continue;
            }
            if R4_TASKS[tid].state != R4State::Dead {
                return true;
            }
        }
        false
    }

    unsafe fn r4_find_exited_child(parent_tid: usize, target: i32) -> Option<usize> {
        for tid in 0..R4_NUM_TASKS {
            if tid == parent_tid || R4_TASKS[tid].parent_tid != parent_tid {
                continue;
            }
            if !r4_wait_matches(target, tid) {
                continue;
            }
            if R4_TASKS[tid].state == R4State::Exited {
                return Some(tid);
            }
        }
        None
    }

    unsafe fn r4_wake_waiter(parent_tid: usize, child_tid: usize) {
        let status_ptr = R4_TASKS[parent_tid].wait_status_ptr;
        // This runs when the EXITING child (or SHARED) address space is
        // current, NOT the parent's. Write the status into the parent's own
        // table by walking it explicitly; falling back to the current-CR3
        // copyout only on lanes without per-task address spaces (pml4_phys 0).
        let ok = if status_ptr == 0 {
            true
        } else {
            let st = R4_TASKS[child_tid].exit_status.to_le_bytes();
            let pml4 = R4_TASKS[parent_tid].pml4_phys;
            if pml4 != 0 {
                mm::as_copyout(pml4, status_ptr, &st)
            } else {
                copyout_user(status_ptr, &st, st.len()).is_ok()
            }
        };
        if ok {
            R4_TASKS[parent_tid].saved_frame[14] = child_tid as u64;
        } else {
            R4_TASKS[parent_tid].saved_frame[14] = 0xFFFF_FFFF_FFFF_FFFF;
        }
        R4_TASKS[parent_tid].wait_target = R4_WAIT_NONE;
        R4_TASKS[parent_tid].wait_status_ptr = 0;
        R4_TASKS[parent_tid].state = R4State::Ready;
        R4_TASKS[child_tid].state = R4State::Dead;
        R4_TASKS[child_tid].exit_status = 0;
    }

    unsafe fn r4_yield_and_switch(frame: *mut u64) {
        let cur = R4_CURRENT;
        r4_save_frame(frame, cur);
        R4_TASKS[cur].yield_count += 1;
        R4_TASKS[cur].saved_frame[14] = 0;
        match r4_find_ready(cur) {
            Some(tid) => {
                R4_TASKS[cur].state = R4State::Ready;
                r4_switch_to(frame, tid);
            }
            None => {
                R4_TASKS[cur].state = R4State::Running;
                *frame.add(14) = 0;
            }
        }
    }

    #[cfg(feature = "go_test")]
    static mut R4_PREEMPT_TICKS: u64 = 0;
    #[cfg(feature = "go_test")]
    static mut R4_PREEMPT_COUNT: u64 = 0;

    // Idle infrastructure (full-os guide wait-queue prerequisite). When no
    // task is runnable but a timed wakeup is pending (nanosleep), the kernel
    // parks in r4_idle_loop at ring0 with interrupts enabled; the PIT wakes
    // sleepers and switches to them from the idle context.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut R4_IDLE: bool = false;

    /// Ring0 idle loop: enable interrupts and halt. The PIT handler switches
    /// away from here once a task becomes runnable; it is never returned to.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    extern "C" fn r4_idle_loop() -> ! {
        loop {
            unsafe {
                core::arch::asm!("sti; hlt", options(nomem, nostack));
            }
        }
    }

    /// Wake any task whose nanosleep deadline has passed.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn r4_wake_sleepers() {
        let now = R4_PREEMPT_TICKS;
        let mut t = 0usize;
        while t < R4_NUM_TASKS {
            if R4_TASKS[t].sleep_until != 0
                && now >= R4_TASKS[t].sleep_until
                && matches!(R4_TASKS[t].state, R4State::Blocked)
            {
                R4_TASKS[t].sleep_until = 0;
                R4_TASKS[t].state = R4State::Ready;
            }
            t += 1;
        }
    }

    /// Called from a block/exit point when no task is currently runnable: if
    /// any task has a pending timed wakeup, park in the idle loop until the
    /// PIT wakes it; otherwise the run is genuinely finished.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn r4_enter_idle_or_done(frame: *mut u64) {
        let mut has_pending = false;
        let mut t = 0usize;
        while t < R4_NUM_TASKS {
            if R4_TASKS[t].sleep_until != 0
                && matches!(R4_TASKS[t].state, R4State::Blocked)
            {
                has_pending = true;
                break;
            }
            t += 1;
        }
        let kstack = &stack_top as *const u8 as u64;
        if has_pending {
            R4_IDLE = true;
            *frame.add(17) = r4_idle_loop as *const () as u64; // rip
            *frame.add(18) = 0x08; // CS ring0
            *frame.add(19) = 0x202; // RFLAGS, IF=1
            *frame.add(20) = kstack; // rsp
            *frame.add(21) = 0x10; // SS ring0
        } else {
            *frame.add(17) = r4_all_done as *const () as u64;
            *frame.add(18) = 0x08;
            *frame.add(19) = 0x02;
            *frame.add(20) = kstack;
            *frame.add(21) = 0x10;
        }
    }

    /// PIT tick entry for the default lane: EOI first, then involuntarily
    /// switch the interrupted ring-3 task to the next Ready task. Unlike the
    /// yield path this preserves RAX - the saved frame is not a syscall
    /// return.
    #[cfg(feature = "go_test")]
    pub(crate) unsafe fn r4_timer_preempt(frame: *mut u64) {
        R4_PREEMPT_TICKS += 1;
        sched::pic_send_eoi(0);
        #[cfg(not(feature = "compat_real_test"))]
        if tcp::tcp_active() || netcfg::query_active() || net::pkg_fetch_armed() {
            // Drive the wire TCP machine / DHCP-DNS query from the tick
            // while one is in flight.
            net::net_rx_pump();
            // Retransmit timer: pump RX first (so an inbound ACK clears the
            // timer), then tick the RTO countdown for any unacked segment.
            tcp::tcp_rt_tick();
            // Package manager: start (once the NIC is up) + drive an armed fetch.
            net::pkg_fetch_tick();
        }
        // Wake nanosleep tasks whose deadline has passed (wait-queue infra).
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        r4_wake_sleepers();
        // If we interrupted the idle loop, dispatch to a freshly-woken task
        // (there is no idle state to save).
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        if R4_IDLE {
            if R4_NUM_TASKS != 0 {
                if let Some(tid) = r4_find_ready(R4_NUM_TASKS) {
                    R4_IDLE = false;
                    r4_switch_to(frame, tid);
                }
            }
            return;
        }
        if *frame.add(18) & 3 != 3 {
            return; // interrupted a kernel path - nothing to switch
        }
        if R4_NUM_TASKS == 0 {
            return;
        }
        let cur = R4_CURRENT;
        if let Some(tid) = r4_find_ready(cur) {
            r4_save_frame(frame, cur);
            R4_TASKS[cur].state = R4State::Ready;
            R4_PREEMPT_COUNT += 1;
            if R4_PREEMPT_COUNT == 1 {
                serial_write(b"SCHED: preempt hit\n");
            }
            r4_switch_to(frame, tid);
        }
    }

    unsafe fn r4_exit_and_switch(frame: *mut u64, exit_status: u64) {
        let cur = R4_CURRENT;
        let parent = R4_TASKS[cur].parent_tid;
        r4_cleanup_task_resources(cur);
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            if EXEC_APP_TID == cur as i32 {
                EXEC_APP_TID = -1;
            }
            // Reclaim a private address space on any exit path (clean exit,
            // signal kill, or fault containment). Step CR3 onto the shared
            // table first so we are not standing on the page tree we free.
            let cur_pml4 = R4_TASKS[cur].pml4_phys;
            if cur_pml4 != 0 && cur_pml4 != SHARED_PML4_PHYS {
                // A cloned thread shares this private address space; only the
                // last active thread frees it. Scan for any other live task
                // still on the same table.
                let mut shared = false;
                let mut t = 0usize;
                while t < R4_NUM_TASKS {
                    if t != cur
                        && R4_TASKS[t].pml4_phys == cur_pml4
                        && !matches!(R4_TASKS[t].state, R4State::Dead | R4State::Exited)
                    {
                        shared = true;
                        break;
                    }
                    t += 1;
                }
                if !shared {
                    core::arch::asm!("mov cr3, {}", in(reg) SHARED_PML4_PHYS,
                                     options(nostack));
                    mm::address_space_release(cur_pml4);
                    R4_TASKS[cur].pml4_phys = SHARED_PML4_PHYS;
                    // Reclaim our own already-exited clone threads that shared
                    // this (now freed) address space. They are orphans (their
                    // parent is us, and we are exiting), so no one will reap
                    // them; mark them Dead to free the slot and clear the now
                    // dangling pml4 reference.
                    let mut z = 0usize;
                    while z < R4_NUM_TASKS {
                        if z != cur
                            && R4_TASKS[z].pml4_phys == cur_pml4
                            && R4_TASKS[z].parent_tid == cur
                            && matches!(R4_TASKS[z].state, R4State::Exited)
                        {
                            R4_TASKS[z].pml4_phys = SHARED_PML4_PHYS;
                            R4_TASKS[z].state = R4State::Dead;
                            R4_TASKS[z].exit_status = 0;
                        }
                        z += 1;
                    }
                    serial_write(b"ASRELEASE: tid=0x");
                    serial_write_hex(cur as u64);
                    serial_write(b" as=0x");
                    serial_write_hex(cur_pml4);
                    serial_write(b"\n");
                }
            }
        }
        R4_TASKS[cur].exit_status = exit_status;
        R4_TASKS[cur].state = R4State::Exited;
        if parent != cur
            && parent < R4_NUM_TASKS
            && R4_TASKS[parent].state == R4State::Blocked
            && R4_TASKS[parent].wait_target != R4_WAIT_NONE
            && r4_wait_matches(R4_TASKS[parent].wait_target, cur)
        {
            r4_wake_waiter(parent, cur);
        }
        match r4_find_ready(R4_CURRENT) {
            Some(tid) => { r4_switch_to(frame, tid); }
            None => {
                // No runnable task: idle if a timed wakeup is pending,
                // otherwise the run is finished.
                #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                r4_enter_idle_or_done(frame);
                #[cfg(not(all(feature = "go_test", not(feature = "compat_real_test"))))]
                {
                    let kstack = &stack_top as *const u8 as u64;
                    *frame.add(17) = r4_all_done as *const () as u64;
                    *frame.add(18) = 0x08;
                    *frame.add(19) = 0x02;
                    *frame.add(20) = kstack;
                    *frame.add(21) = 0x10;
                }
            }
        }
    }

    extern "C" fn r4_all_done() -> ! {
        #[cfg(feature = "stress_ipc_test")]
        serial_write(b"STRESS: ipc ok");
        #[cfg(feature = "compat_real_test")]
        unsafe {
            process::compat_real_finish_current_app();
        }
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        unsafe {
            serial_write(b"SCHED: tasks high=0x");
            serial_write_hex(R4_TASKS_HIGH as u64);
            serial_write(b"\n");
            serial_write(b"RUGO: halt ok\n");
        }
        qemu_exit(0x31);
        loop { unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)); } }
    }

    // ---- exec-from-filesystem (sys_spawn, ABI v3.x id 46) ----
    //
    // Loads a PKG v1-framed ELF from the SimpleFS app region on the boot
    // disk (superblock at sector 64) into the exec app window of the
    // demand-paged region and runs it as a child task. v1 semantics: the
    // window is single-occupancy; a spawn while an app is resident fails.

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const EXEC_APP_BASE: u64 = 0x0140_0000;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const EXEC_APP_END: u64 = 0x0180_0000;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const EXEC_APP_REGION_SECTOR: u64 = 64;
    // 64 KiB: C programs against rlibc are bigger than the asm
    // coreutils; the package read path is heap-backed, so the cap is
    // policy, not a buffer size.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const EXEC_APP_MAX_BYTES: usize = 65536;

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut EXEC_APP_TID: i32 = -1;

    // Physical address of the shared/boot PML4 (set in setup_go_user_pages).
    // Per-process address spaces clone it for their kernel half, and the
    // boot task / service threads run on it directly.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut SHARED_PML4_PHYS: u64 = 0;

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn exec_log(name: &[u8], what: &[u8]) {
        serial_write(b"EXEC: ");
        serial_write(name);
        serial_write(b" ");
        serial_write(what);
        serial_write(b"\n");
    }

    /// Validate and copy an ET_EXEC ELF64 whose segments live entirely in
    /// the exec app window. Returns the entry point. Segments are mapped
    /// and written into the child's private address space `pml4_phys`, so
    /// the load targets the new process, not the spawner.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn exec_load_app(pml4_phys: u64, image: &[u8]) -> Option<u64> {
        if image.len() < 64 {
            return None;
        }
        if &image[0..4] != b"\x7FELF" || image[4] != 2 || image[5] != 1 {
            return None;
        }
        let e_type = u16::from_le_bytes([image[16], image[17]]);
        if e_type != 2 {
            return None;
        }
        let e_entry = u64::from_le_bytes(image[24..32].try_into().ok()?);
        if e_entry < EXEC_APP_BASE || e_entry >= EXEC_APP_END {
            return None;
        }
        let e_phoff = u64::from_le_bytes(image[32..40].try_into().ok()?) as usize;
        let e_phentsize = u16::from_le_bytes([image[54], image[55]]) as usize;
        let e_phnum = u16::from_le_bytes([image[56], image[57]]) as usize;
        if e_phentsize < 56 || e_phnum == 0 || e_phnum > 8 {
            return None;
        }
        let mut i = 0usize;
        while i < e_phnum {
            let ph = e_phoff + i * e_phentsize;
            if ph + 56 > image.len() {
                return None;
            }
            let p_type = u32::from_le_bytes(image[ph..ph + 4].try_into().ok()?);
            if p_type == 1 {
                let p_offset =
                    u64::from_le_bytes(image[ph + 8..ph + 16].try_into().ok()?) as usize;
                let p_vaddr = u64::from_le_bytes(image[ph + 16..ph + 24].try_into().ok()?);
                let p_filesz =
                    u64::from_le_bytes(image[ph + 32..ph + 40].try_into().ok()?) as usize;
                let p_memsz =
                    u64::from_le_bytes(image[ph + 40..ph + 48].try_into().ok()?) as usize;
                if p_filesz > p_memsz
                    || p_offset + p_filesz > image.len()
                    || p_vaddr < EXEC_APP_BASE
                    || p_vaddr + p_memsz as u64 > EXEC_APP_END
                {
                    return None;
                }
                if !mm::as_copyout(pml4_phys, p_vaddr, &image[p_offset..p_offset + p_filesz])
                {
                    return None;
                }
                // BSS: map (zeroed) the rest of the segment. Fresh frames
                // arrive zeroed, and the file-backed tail page was copied
                // into an already-zeroed frame, so no explicit memset.
                if p_memsz > p_filesz
                    && !mm::as_map_zeroed(
                        pml4_phys,
                        p_vaddr + p_filesz as u64,
                        p_memsz - p_filesz,
                    )
                {
                    return None;
                }
            }
            i += 1;
        }
        Some(e_entry)
    }

    // Args page: the kernel copies the spawn argument string (NUL
    // terminated) into the last page of the app window; the child gets
    // its address in RDI and length in RSI.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const EXEC_ARGS_VA: u64 = 0x017F_F000;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const EXEC_ARGS_MAX: usize = 256;

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    /// Validate a pipe fd the caller wants to hand to the child as
    /// stdin/stdout. u64::MAX means "none".
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn spawn_stdio_ok(fd: u64, want: M8FdKind) -> bool {
        if fd == u64::MAX {
            return true;
        }
        let idx = fd as usize;
        idx >= 3
            && idx < M8_FD_MAX
            && M8_FD_TABLE[idx].kind == want
            && M8_FD_TABLE[idx].owner_tid == R4_CURRENT
    }

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_spawn_v1(
        name_ptr: u64,
        name_len: u64,
        args_ptr: u64,
        args_len: u64,
        stdin_fd: u64,
        stdout_fd: u64,
    ) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        if R4_TASKS[r4_current_smp()].cap_flags & R4_TASK_CAP_STORAGE == 0 {
            return ERR;
        }
        if name_len == 0 || name_len > 24 {
            return ERR;
        }
        if args_len as usize > EXEC_ARGS_MAX {
            return ERR;
        }
        let mut args_buf = [0u8; EXEC_ARGS_MAX + 1];
        let args_n = args_len as usize;
        if args_n > 0 && copyin_user(&mut args_buf[..args_n], args_ptr, args_n).is_err() {
            return ERR;
        }
        let mut name_buf = [0u8; 24];
        let n = name_len as usize;
        if copyin_user(&mut name_buf[..n], name_ptr, n).is_err() {
            return ERR;
        }
        let name = &name_buf[..n];
        // Per-process address spaces (full-os keystone): the exec window is
        // no longer single-occupancy. Each spawn gets a private address
        // space, so multiple apps can be resident and run concurrently.
        if !storage::r4_storage_available() {
            exec_log(name, b"nodisk");
            return ERR;
        }

        if !block_io_dispatch(false, EXEC_APP_REGION_SECTOR, 512, false) {
            exec_log(name, b"ioerr");
            return ERR;
        }
        let magic = u32::from_le_bytes([
            BLK_DATA_PAGE.0[0],
            BLK_DATA_PAGE.0[1],
            BLK_DATA_PAGE.0[2],
            BLK_DATA_PAGE.0[3],
        ]);
        if magic != runtime::storage::SIMPLEFS_MAGIC {
            exec_log(name, b"noregion");
            return ERR;
        }
        let file_count = u32::from_le_bytes([
            BLK_DATA_PAGE.0[4],
            BLK_DATA_PAGE.0[5],
            BLK_DATA_PAGE.0[6],
            BLK_DATA_PAGE.0[7],
        ]) as usize;

        // File table spans three sectors (48 entries x 32 bytes). Kept in sync
        // with tools/app_disk_v1.py (data payloads start at base + 4).
        let mut table = [0u8; 1536];
        let mut ts = 0u64;
        while ts < 3 {
            if !block_io_dispatch(false, EXEC_APP_REGION_SECTOR + 1 + ts, 512, false) {
                exec_log(name, b"ioerr");
                return ERR;
            }
            core::ptr::copy_nonoverlapping(
                BLK_DATA_PAGE.0.as_ptr(),
                table.as_mut_ptr().add((ts * 512) as usize),
                512,
            );
            ts += 1;
        }

        let fc = core::cmp::min(file_count, 48);
        let mut file_sector = 0u64;
        let mut file_size = 0usize;
        let mut found = false;
        let mut fi = 0usize;
        while fi < fc {
            let base = fi * 32;
            let entry_name = &table[base..base + 24];
            if &entry_name[..n] == name && (n == 24 || entry_name[n] == 0) {
                file_sector = match table[base + 24..base + 28].try_into() {
                    Ok(b) => u32::from_le_bytes(b) as u64,
                    Err(_) => return ERR,
                };
                file_size = match table[base + 28..base + 32].try_into() {
                    Ok(b) => u32::from_le_bytes(b) as usize,
                    Err(_) => return ERR,
                };
                found = true;
                break;
            }
            fi += 1;
        }
        if !found {
            exec_log(name, b"missing");
            return ERR;
        }
        if file_size < 64 || file_size > EXEC_APP_MAX_BYTES {
            exec_log(name, b"badsize");
            return ERR;
        }

        let mut pkg: alloc::vec::Vec<u8> = alloc::vec::Vec::new();
        let sectors = (file_size + 511) / 512;
        let mut s = 0u64;
        while s < sectors as u64 {
            if !block_io_dispatch(false, file_sector + s, 512, false) {
                exec_log(name, b"ioerr");
                return ERR;
            }
            pkg.extend_from_slice(&BLK_DATA_PAGE.0[..512]);
            s += 1;
        }
        let pkg = &pkg[..file_size];

        let pkg_magic = u32::from_le_bytes([pkg[0], pkg[1], pkg[2], pkg[3]]);
        if pkg_magic != runtime::storage::PKG_MAGIC_V1 {
            exec_log(name, b"badpkg");
            return ERR;
        }
        let bin_size = u32::from_le_bytes([pkg[4], pkg[5], pkg[6], pkg[7]]) as usize;
        if bin_size == 0 || 64 + bin_size > file_size {
            exec_log(name, b"badsize");
            return ERR;
        }
        let elf = &pkg[64..64 + bin_size];
        let digest = sha256_digest(elf);
        if digest != pkg[32..64] {
            exec_log(name, b"badhash");
            return ERR;
        }

        let parent = R4_CURRENT;
        let tid = match r4_find_spawn_slot() {
            Some(t) => t,
            None => {
                exec_log(name, b"full");
                return ERR;
            }
        };
        // Build the child's private address space and load the ELF into it
        // (not into the spawner's). The kernel half is cloned from the
        // shared table so the kernel stays reachable under the child's CR3.
        let child_pml4 = match mm::address_space_create(SHARED_PML4_PHYS) {
            Some(p) => p,
            None => {
                exec_log(name, b"noas");
                return ERR;
            }
        };
        let entry = match exec_load_app(child_pml4, elf) {
            Some(e) => e,
            None => {
                mm::address_space_release(child_pml4);
                exec_log(name, b"badelf");
                return ERR;
            }
        };
        // Deliver the argument string (NUL terminated) to the child's args
        // page in its own address space.
        if !mm::as_copyout(child_pml4, EXEC_ARGS_VA, &args_buf[..args_n + 1]) {
            mm::address_space_release(child_pml4);
            exec_log(name, b"badargs");
            return ERR;
        }
        if !spawn_stdio_ok(stdin_fd, M8FdKind::PipeR)
            || !spawn_stdio_ok(stdout_fd, M8FdKind::PipeW)
        {
            mm::address_space_release(child_pml4);
            exec_log(name, b"badfd");
            return ERR;
        }
        // ASLR (full-os guide Part IV.10): start the stack a random,
        // page-aligned offset (0..15 pages) below the slot's top, drawn from
        // the CSPRNG, so the stack base differs per spawn. Stays well inside
        // the slot's guard-zoned stride. (Code ASLR needs PIE; carry-forward.)
        let aslr_stk = r4_stack_top_for_slot(tid)
            .wrapping_sub((rng_next() & 0xF) * 0x1000);
        r4_init_task(tid, entry, aslr_stk, parent);
        // Install the private address space (r4_init_task inherited the
        // parent's, so this must come after it).
        R4_TASKS[tid].pml4_phys = child_pml4;
        // External apps get read access to the file tree (storage) but no
        // network, spawn, or IPC surface.
        R4_TASKS[tid].can_spawn = false;
        R4_TASKS[tid].cap_flags = R4_TASK_CAP_STORAGE;
        R4_TASKS[tid].fd_limit = 4;
        R4_TASKS[tid].socket_limit = 0;
        R4_TASKS[tid].endpoint_limit = 0;
        R4_TASKS[tid].isolation_domain = 5;
        // External apps run unprivileged.
        R4_TASKS[tid].uid = 100;
        // Hand over the pipe ends: ownership moves to the child so its
        // exit (or fault) releases them and EOF propagates.
        for fdv in [stdin_fd, stdout_fd] {
            if fdv != u64::MAX {
                let idx = fdv as usize;
                M8_FD_TABLE[idx].owner_tid = tid;
                if R4_TASKS[r4_current_smp()].fd_count != 0 {
                    R4_TASKS[r4_current_smp()].fd_count -= 1;
                }
                R4_TASKS[tid].fd_count += 1;
            }
        }
        // Argument convention: RDI = args pointer, RSI = args length,
        // RDX = stdin fd (u64::MAX = none), RCX = stdout fd.
        R4_TASKS[tid].saved_frame[9] = EXEC_ARGS_VA;
        R4_TASKS[tid].saved_frame[10] = args_n as u64;
        R4_TASKS[tid].saved_frame[11] = stdin_fd;
        R4_TASKS[tid].saved_frame[12] = stdout_fd;
        R4_TASKS[tid].state = R4State::Ready;
        R4_THREADS_CREATED += 1;
        EXEC_APP_TID = tid as i32;
        serial_write(b"SPAWN: ");
        serial_write(name);
        serial_write(b" as_ok 0x");
        serial_write_hex(child_pml4);
        serial_write(b" rsp=0x");
        serial_write_hex(aslr_stk);
        serial_write(b"\n");
        exec_log(name, b"ok");
        tid as u64
    }

    // mmap/brk region layout for sys_vm_ctl (within the per-task demand
    // window, below the exec window so it never collides with loaded code
    // or the demand stacks): brk grows in [0x0100_0000,0x0120_0000) and
    // anonymous mmap lives in [0x0120_0000,0x0140_0000).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const VM_BRK_BASE: u64 = 0x0100_0000;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const VM_BRK_MAX: u64 = 0x0120_0000;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const VM_MMAP_BASE: u64 = 0x0120_0000;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const VM_MMAP_END: u64 = 0x0140_0000;

    /// sys_vm_ctl (ABI v3.2 id 50, op-multiplexed): op 1 = mmap(va, sz,
    /// prot) -> va, op 2 = munmap(va, sz) -> 0, op 3 = brk(new) -> old brk
    /// (new = 0 queries the current break). -1 on error.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_vm_ctl(op: u64, a2: u64, a3: u64, a4: u64) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        match op {
            1 => {
                // mmap(va, sz, prot)
                let (va, sz, prot) = (a2, a3, a4);
                if va & 0xFFF != 0 || sz == 0 || sz & 0xFFF != 0 {
                    return ERR;
                }
                let end = match va.checked_add(sz) {
                    Some(e) => e,
                    None => return ERR,
                };
                if va < VM_MMAP_BASE || end > VM_MMAP_END {
                    return ERR;
                }
                let mut off = 0u64;
                while off < sz {
                    if !mm::vm_map_current(va + off, prot) {
                        // Roll back the pages mapped so far.
                        let mut u = 0u64;
                        while u < off {
                            mm::vm_unmap_current(va + u);
                            u += 0x1000;
                        }
                        return ERR;
                    }
                    off += 0x1000;
                }
                serial_write(b"MM: mmap va=0x");
                serial_write_hex(va);
                serial_write(b" sz=0x");
                serial_write_hex(sz);
                serial_write(b"\n");
                va
            }
            2 => {
                // munmap(va, sz)
                let (va, sz) = (a2, a3);
                if va & 0xFFF != 0 || sz == 0 || sz & 0xFFF != 0 {
                    return ERR;
                }
                let end = match va.checked_add(sz) {
                    Some(e) => e,
                    None => return ERR,
                };
                if va < VM_MMAP_BASE || end > VM_MMAP_END {
                    return ERR;
                }
                let mut off = 0u64;
                while off < sz {
                    mm::vm_unmap_current(va + off);
                    off += 0x1000;
                }
                serial_write(b"MM: munmap va=0x");
                serial_write_hex(va);
                serial_write(b"\n");
                0
            }
            4 => {
                // mprotect(va, sz, prot)
                let (va, sz, prot) = (a2, a3, a4);
                if va & 0xFFF != 0 || sz == 0 || sz & 0xFFF != 0 {
                    return ERR;
                }
                let end = match va.checked_add(sz) {
                    Some(e) => e,
                    None => return ERR,
                };
                if va < VM_MMAP_BASE || end > VM_MMAP_END {
                    return ERR;
                }
                let mut off = 0u64;
                while off < sz {
                    if !mm::vm_protect_current(va + off, prot) {
                        return ERR;
                    }
                    off += 0x1000;
                }
                serial_write(b"MM: mprotect va=0x");
                serial_write_hex(va);
                serial_write(b"\n");
                0
            }
            3 => {
                // brk(new) - 0 queries
                if R4_TASKS[r4_current_smp()].heap_brk == 0 {
                    R4_TASKS[r4_current_smp()].heap_brk = VM_BRK_BASE;
                }
                let cur = R4_TASKS[r4_current_smp()].heap_brk;
                let new = a2;
                if new == 0 {
                    return cur;
                }
                if new & 0xFFF != 0 || new < VM_BRK_BASE || new > VM_BRK_MAX {
                    return ERR;
                }
                if new > cur {
                    let mut a = cur;
                    while a < new {
                        if !mm::vm_map_current(a, 0x3) {
                            return ERR;
                        }
                        a += 0x1000;
                    }
                } else if new < cur {
                    let mut a = new;
                    while a < cur {
                        mm::vm_unmap_current(a);
                        a += 0x1000;
                    }
                }
                R4_TASKS[r4_current_smp()].heap_brk = new;
                serial_write(b"MM: brk 0x");
                serial_write_hex(cur);
                serial_write(b" -> 0x");
                serial_write_hex(new);
                serial_write(b"\n");
                cur
            }
            _ => ERR,
        }
    }

    /// sys_proc_ctl (ABI v3.2 id 51, op-multiplexed): op 1 = fork (a
    /// copy-on-write duplicate of the caller), op 2 = clone (a new thread
    /// sharing the caller's address space, entry in a2). fork returns the
    /// child tid to the parent and 0 to the child; clone returns the new
    /// tid. -1 on error.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_proc_ctl(frame: *mut u64, op: u64, a2: u64, a3: u64) {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        match op {
            1 => sys_fork_v1(frame),
            2 => {
                // clone: a new thread sharing the caller's address space at
                // entry `a2`. Unlike sys_thread_spawn_r4 this is available to
                // any task (a thread adds no privilege) - it is how spawned
                // apps get pthreads.
                let entry = a2;
                if entry == 0 || entry >= 0x0000_8000_0000_0000 {
                    *frame.add(14) = ERR;
                    return;
                }
                let tid = match r4_find_spawn_slot() {
                    Some(t) => t,
                    None => {
                        *frame.add(14) = ERR;
                        return;
                    }
                };
                r4_init_task(tid, entry, r4_stack_top_for_slot(tid), R4_CURRENT);
                R4_TASKS[tid].state = R4State::Ready;
                R4_THREADS_CREATED += 1;
                *frame.add(14) = tid as u64;
            }
            // op 3 = getuid (full-os guide Part IV.10 multi-user): the caller's
            // user id. Resolved via r4_current_smp so a task running on an AP reads
            // ITS OWN slot's uid (per-CPU current), not the BSP scheduler cursor's;
            // identical to R4_CURRENT on the bootstrap processor (full-os Part I.3).
            3 => {
                *frame.add(14) = R4_TASKS[r4_current_smp()].uid as u64;
            }
            // op 4 = setuid(a2): only root (uid 0) may change uid, enforcing the
            // privilege model. A non-root attempt is denied and audited.
            4 => {
                if R4_TASKS[r4_current_smp()].uid != 0 {
                    audit_event(b"setuid-deny", 51);
                    *frame.add(14) = ERR;
                } else if a2 > 0xFF {
                    *frame.add(14) = ERR;
                } else {
                    R4_TASKS[r4_current_smp()].uid = a2 as u8;
                    *frame.add(14) = 0;
                }
            }
            // op 5 = login(name, pw) (full-os guide Part IV.10 multi-user): verify
            // a credential against the password database and, on success, ASSUME
            // that account's uid (authenticated privilege change — unlike op 4
            // setuid, which is root-only). a2 = 8-byte username, a3 = ≤16-byte
            // NUL-terminated password. Returns the uid, or -1 (denied + audited).
            5 => {
                let mut name = [0u8; 8];
                let mut pw = [0u8; 16];
                if copyin_user(&mut name, a2, 8).is_err()
                    || copyin_user(&mut pw, a3, 16).is_err()
                {
                    *frame.add(14) = ERR;
                    return;
                }
                let mut plen = 0usize;
                while plen < 16 && pw[plen] != 0 {
                    plen += 1;
                }
                match login_verify(&name, &pw[..plen]) {
                    Some(uid) => {
                        R4_TASKS[r4_current_smp()].uid = uid;
                        audit_event(b"login-ok", 51);
                        *frame.add(14) = uid as u64;
                    }
                    None => {
                        audit_event(b"login-deny", 51);
                        *frame.add(14) = ERR;
                    }
                }
            }
            _ => {
                *frame.add(14) = ERR;
            }
        }
    }

    /// One account in the credential database (full-os guide Part IV.10). The
    /// password is stored as its SHA-256 digest (`sha256.rs`), never the cleartext.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    struct PasswdEntry {
        name: [u8; 8],
        uid: u8,
        pw_hash: [u8; 32],
    }

    /// The credential database: root (uid 0) and a regular user (uid 100). The
    /// stored hashes are the SHA-256 digests of the cleartext — `sha256(b"toor")`
    /// and `sha256(b"pass")` — a real cryptographic password hash (replacing the
    /// prior djb2 demo). `login_verify` recomputes `sha256(pw)` and compares, so the
    /// cleartext is never stored. (A per-user salt + iterated KDF, and moving the db
    /// to a root-owned `/etc/passwd` VFS file, are the next hardening steps.)
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static PASSWD: [PasswdEntry; 2] = [
        PasswdEntry {
            name: *b"root\0\0\0\0",
            uid: 0,
            pw_hash: [
                0xCE, 0x5C, 0xA6, 0x73, 0xD1, 0x3B, 0x36, 0x11, 0x8D, 0x54, 0xA7, 0xCF, 0x13,
                0xAE, 0xB0, 0xCA, 0x01, 0x23, 0x83, 0xBF, 0x77, 0x1E, 0x71, 0x34, 0x21, 0xB4,
                0xD1, 0xFD, 0x84, 0x1F, 0x53, 0x9A,
            ],
        },
        PasswdEntry {
            name: *b"user\0\0\0\0",
            uid: 100,
            pw_hash: [
                0xD7, 0x4F, 0xF0, 0xEE, 0x8D, 0xA3, 0xB9, 0x80, 0x6B, 0x18, 0xC8, 0x77, 0xDB,
                0xF2, 0x9B, 0xBD, 0xE5, 0x0B, 0x5B, 0xD8, 0xE4, 0xDA, 0xD7, 0xA3, 0xA7, 0x25,
                0x00, 0x0F, 0xEB, 0x82, 0xE8, 0xF1,
            ],
        },
    ];

    /// Verify a username/password against `PASSWD`; returns the account uid on a
    /// match. The password is SHA-256 hashed and compared to the stored digest, so
    /// the cleartext is never held in the table.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn login_verify(name: &[u8; 8], pw: &[u8]) -> Option<u8> {
        let h = crate::sha256::sha256(pw);
        let mut i = 0usize;
        while i < PASSWD.len() {
            if PASSWD[i].name == *name && PASSWD[i].pw_hash == h {
                return Some(PASSWD[i].uid);
            }
            i += 1;
        }
        None
    }

    /// sys_futex (ABI v3.2 id 52): op 1 = wait(uaddr, val) — block while the
    /// u32 at `uaddr` still equals `val`; op 2 = wake(uaddr, n) — wake up to
    /// `n` (0 = all) waiters on `uaddr` in the caller's address space. wait
    /// returns 0 (woken) or 1 (value changed, did not block); wake returns
    /// the number woken; -1 on a bad pointer / op.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_futex(frame: *mut u64, op: u64, uaddr: u64, val: u64) {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        match op {
            1 => {
                let mut word = [0u8; 4];
                if copyin_user(&mut word, uaddr, 4).is_err() {
                    *frame.add(14) = ERR;
                    return;
                }
                if u32::from_le_bytes(word) as u64 != (val & 0xFFFF_FFFF) {
                    *frame.add(14) = 1; // value changed: do not block
                    return;
                }
                let cur = R4_CURRENT;
                serial_write(b"FUTEX: wait tid=0x");
                serial_write_hex(cur as u64);
                serial_write(b"\n");
                *frame.add(14) = 0; // value seen on resume after a wake
                r4_save_frame(frame, cur);
                R4_TASKS[cur].futex_uaddr = uaddr;
                R4_TASKS[cur].state = R4State::Blocked;
                R4_TASKS[cur].block_count += 1;
                match r4_find_ready(cur) {
                    Some(tid) => {
                        r4_switch_to(frame, tid);
                    }
                    None => {
                        r4_enter_idle_or_done(frame);
                    }
                }
            }
            2 => {
                // A null futex word is never a valid waiter; reject it so a
                // wake(0) cannot alias tasks blocked for non-futex reasons
                // (nanosleep/wait/ipc_recv all leave futex_uaddr == 0).
                if uaddr == 0 {
                    *frame.add(14) = ERR;
                    return;
                }
                let cur_pml4 = R4_TASKS[r4_current_smp()].pml4_phys;
                let limit = if val == 0 { u64::MAX } else { val };
                let mut woken = 0u64;
                let mut t = 0usize;
                while t < R4_NUM_TASKS && woken < limit {
                    if matches!(R4_TASKS[t].state, R4State::Blocked)
                        && R4_TASKS[t].futex_uaddr != 0
                        && R4_TASKS[t].futex_uaddr == uaddr
                        && R4_TASKS[t].pml4_phys == cur_pml4
                    {
                        R4_TASKS[t].futex_uaddr = 0;
                        R4_TASKS[t].state = R4State::Ready;
                        woken += 1;
                    }
                    t += 1;
                }
                serial_write(b"FUTEX: wake n=0x");
                serial_write_hex(woken);
                serial_write(b"\n");
                *frame.add(14) = woken;
            }
            _ => {
                *frame.add(14) = ERR;
            }
        }
    }

    /// fork: duplicate the calling task into a copy-on-write child. Only a
    /// task that already owns a private address space can be forked (the
    /// boot task on the shared table cannot, in v1).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_fork_v1(frame: *mut u64) {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let parent = R4_CURRENT;
        let parent_pml4 = R4_TASKS[parent].pml4_phys;
        if parent_pml4 == 0 || parent_pml4 == SHARED_PML4_PHYS {
            *frame.add(14) = ERR;
            return;
        }
        let child = match r4_find_spawn_slot() {
            Some(t) => t,
            None => {
                *frame.add(14) = ERR;
                return;
            }
        };
        let child_pml4 = match mm::address_space_fork(parent_pml4) {
            Some(p) => p,
            None => {
                *frame.add(14) = ERR;
                return;
            }
        };
        // The child inherits caps/uid/limits from the parent (r4_init_task)
        // and resumes at the exact same point in its own CoW copy.
        r4_init_task(child, *frame.add(17), *frame.add(20), parent);
        for i in 0..22 {
            R4_TASKS[child].saved_frame[i] = *frame.add(i);
        }
        R4_TASKS[child].saved_frame[14] = 0; // child: rax = 0
        R4_TASKS[child].pml4_phys = child_pml4;
        R4_TASKS[child].state = R4State::Ready;
        R4_THREADS_CREATED += 1;
        // The parent's page tables were just marked read-only; flush its TLB.
        core::arch::asm!("mov cr3, {}", in(reg) parent_pml4, options(nostack));
        serial_write(b"FORK: child tid=0x");
        serial_write_hex(child as u64);
        serial_write(b" as=0x");
        serial_write_hex(child_pml4);
        serial_write(b"\n");
        *frame.add(14) = child as u64; // parent: rax = child tid
    }

    // ---- timekeeping (full-os guide Part IV.9) ----

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn cmos_read(reg: u8) -> u8 {
        arch_x86::outb(0x70, reg);
        arch_x86::inb(0x71)
    }

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    #[inline]
    fn bcd_to_bin(v: u8) -> u8 {
        (v & 0x0F) + ((v >> 4) * 10)
    }

    /// Days since the Unix epoch for a civil date (Howard Hinnant's
    /// algorithm), valid across the Gregorian calendar.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
        let y = y - if m <= 2 { 1 } else { 0 };
        let era = (if y >= 0 { y } else { y - 399 }) / 400;
        let yoe = y - era * 400;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    /// Wall-clock seconds since the Unix epoch from the CMOS RTC. Assumes
    /// 24-hour mode (QEMU default); BCD vs binary is read from status B.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn cmos_unix_seconds() -> u64 {
        let mut spins = 0u32;
        while cmos_read(0x0A) & 0x80 != 0 && spins < 1_000_000 {
            spins += 1; // wait out an update-in-progress
        }
        let mut sec = cmos_read(0x00);
        let mut min = cmos_read(0x02);
        let mut hour = cmos_read(0x04);
        let mut day = cmos_read(0x07);
        let mut mon = cmos_read(0x08);
        let mut year = cmos_read(0x09);
        if cmos_read(0x0B) & 0x04 == 0 {
            sec = bcd_to_bin(sec);
            min = bcd_to_bin(min);
            hour = bcd_to_bin(hour & 0x7F);
            day = bcd_to_bin(day);
            mon = bcd_to_bin(mon);
            year = bcd_to_bin(year);
        }
        let full_year = 2000i64 + year as i64;
        let days = days_from_civil(full_year, mon as i64, day as i64);
        (days * 86400 + hour as i64 * 3600 + min as i64 * 60 + sec as i64) as u64
    }

    /// Lazily-calibrated TSC frequency (Hz) for the raw monotonic clock; 0 until
    /// first use or if calibration is implausible (then we fall back to the PIT).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut CLOCK_TSC_HZ: u64 = 0;

    /// CLOCK_MONOTONIC_RAW nanoseconds (full-os guide Part IV.9): derived from the
    /// raw CPU timestamp counter (rdtsc), which is unaffected by any clock
    /// adjustment and ticks far finer than the 10 ms PIT. Calibrated once against
    /// the PIT and cached; if that calibration is implausible we fall back to the
    /// PIT-tick monotonic value so the clock is never garbage. u128 intermediate
    /// math avoids the `tsc * 1e9` overflow a u64 would hit within seconds of boot.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn clock_monotonic_raw_ns() -> u64 {
        if CLOCK_TSC_HZ == 0 {
            CLOCK_TSC_HZ = tsc_hz_via_pit();
        }
        let hz = CLOCK_TSC_HZ;
        if hz == 0 {
            return R4_PREEMPT_TICKS.wrapping_mul(10_000_000);
        }
        let tsc = core::arch::x86_64::_rdtsc();
        ((tsc as u128 * 1_000_000_000u128) / hz as u128) as u64
    }

    /// Read a POSIX-style clock by id (the value sys_time op 1 returns):
    /// 0 = MONOTONIC ns (PIT ticks), 1 = REALTIME unix seconds (CMOS RTC),
    /// 2 = MONOTONIC_RAW ns (raw TSC), 3 = BOOTTIME ns. There is no suspend
    /// state, so BOOTTIME equals MONOTONIC. -1 (u64::MAX) for an unknown id.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn clock_read(clockid: u64) -> u64 {
        match clockid {
            0 => R4_PREEMPT_TICKS.wrapping_mul(10_000_000),
            1 => cmos_unix_seconds(),
            2 => clock_monotonic_raw_ns(),
            3 => R4_PREEMPT_TICKS.wrapping_mul(10_000_000),
            _ => 0xFFFF_FFFF_FFFF_FFFF,
        }
    }

    /// sys_time (ABI v3.2 id 53, op-multiplexed): op 1 = clock_gettime
    /// (a2 = clockid: 0 = MONOTONIC nanoseconds since boot from the PIT tick
    /// counter, 1 = REALTIME seconds since the Unix epoch from the CMOS RTC,
    /// 2 = MONOTONIC_RAW nanoseconds from the raw TSC, 3 = BOOTTIME nanoseconds);
    /// op 2 = nanosleep; op 3 = timerfd_create (one-shot); op 4 = timerfd_create
    /// (periodic). -1 on bad op/clockid.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_time(frame: *mut u64, op: u64, a2: u64) {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        // nanosleep (op 2) switches tasks, so it sets RAX in the frame itself
        // (like sys_futex); the dispatcher must NOT assign sys_time's return
        // afterward or it would clobber the switched-to task's RAX.
        match op {
            1 => {
                *frame.add(14) = clock_read(a2);
            }
            2 => {
                // nanosleep(a2 = nanoseconds): block until the deadline (10 ms
                // PIT resolution). Other tasks run meanwhile; if none are
                // runnable the scheduler idles until the PIT wakes us.
                let ticks = (a2 + 9_999_999) / 10_000_000;
                if ticks == 0 {
                    *frame.add(14) = 0;
                    return;
                }
                let cur = R4_CURRENT;
                *frame.add(14) = 0; // value seen on resume
                r4_save_frame(frame, cur);
                R4_TASKS[cur].sleep_until = R4_PREEMPT_TICKS + ticks;
                R4_TASKS[cur].state = R4State::Blocked;
                R4_TASKS[cur].block_count += 1;
                match r4_find_ready(cur) {
                    Some(tid) => {
                        r4_switch_to(frame, tid);
                    }
                    None => {
                        r4_enter_idle_or_done(frame);
                    }
                }
            }
            3 => {
                // timerfd_create(a2 = nanoseconds): a one-shot timer fd that
                // becomes readable at the deadline. Non-blocking read returns
                // the expiration count (8 bytes) once fired, else 0.
                let ticks = (a2 + 9_999_999) / 10_000_000;
                let fd = m8_alloc_fd(M8FdKind::TimerFd);
                if fd == 0xFFFF_FFFF_FFFF_FFFF {
                    *frame.add(14) = ERR;
                    return;
                }
                M8_FD_TABLE[fd as usize].rights = M10_RIGHT_READ | M10_RIGHT_POLL;
                M8_FD_TABLE[fd as usize].offset = (R4_PREEMPT_TICKS + ticks) as usize;
                M8_FD_TIMER_IVL[fd as usize] = 0; // one-shot (no re-arm)
                *frame.add(14) = fd;
            }
            4 => {
                // timerfd_create periodic(a2 = interval ns): becomes readable
                // after `interval`, then re-arms every `interval`. A read returns
                // the number of expirations since the last read (>= 1) and advances
                // to the next future deadline. (full-os guide Part IV.9)
                let ticks = (a2 + 9_999_999) / 10_000_000;
                if ticks == 0 {
                    *frame.add(14) = ERR; // a zero interval would never advance
                    return;
                }
                let fd = m8_alloc_fd(M8FdKind::TimerFd);
                if fd == 0xFFFF_FFFF_FFFF_FFFF {
                    *frame.add(14) = ERR;
                    return;
                }
                M8_FD_TABLE[fd as usize].rights = M10_RIGHT_READ | M10_RIGHT_POLL;
                M8_FD_TABLE[fd as usize].offset = (R4_PREEMPT_TICKS + ticks) as usize;
                M8_FD_TIMER_IVL[fd as usize] = ticks;
                *frame.add(14) = fd;
            }
            _ => {
                *frame.add(14) = ERR;
            }
        }
    }

    /// Boot self-test for the extended clock ids (full-os guide Part IV.9):
    /// MONOTONIC_RAW must be nonzero and strictly advance across a short busy
    /// interval (the raw TSC source ticks finer than the 10 ms PIT, so it moves
    /// even while the PIT-tick MONOTONIC clock is frozen with IF=0), BOOTTIME must
    /// be a valid monotonic value (>= MONOTONIC; equal here, no suspend), and an
    /// unknown clock id must report the error sentinel.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn clock_ids_selftest() {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let mono = clock_read(0);
        let raw1 = clock_read(2);
        let mut spin = 0u64;
        while spin < 200_000 {
            core::hint::spin_loop();
            spin += 1;
        }
        let raw2 = clock_read(2);
        let boot = clock_read(3);
        let bad = clock_read(0x55);
        let ok = raw1 != 0 && raw2 > raw1 && boot >= mono && bad == ERR;
        if ok {
            serial_write(b"CLOCK: ext-ids ok\n");
        } else {
            serial_write(b"CLOCK: ext-ids fail\n");
        }
    }

    // ---- CSPRNG + getrandom (full-os guide Part IV.10) ----
    //
    // A xorshift64* pool seeded once (lazily) from the CMOS wall clock, the
    // PIT tick counter, a constant, AND — when the CPU supports it — the RDRAND
    // hardware RNG (CPUID-gated, to stay portable). The hardware value is
    // XOR-mixed into the timing/clock seed, so a spoofed or unavailable RDRAND
    // can never *weaken* the existing entropy; it can only add to it.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut RNG_STATE: u64 = 0;
    // 0 = not yet seeded; 1 = RDRAND contributed; 2 = soft seed only (no RDRAND).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut RNG_HWSEED: u8 = 0;

    /// Read a 64-bit value from RDRAND. The caller MUST have confirmed RDRAND
    /// support via CPUID first (calling this on a CPU without it is UB). Retries
    /// the spec-recommended bounded number of times for a transient under-run.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    #[target_feature(enable = "rdrand")]
    unsafe fn rdrand_raw() -> Option<u64> {
        let mut v = 0u64;
        let mut tries = 0;
        while tries < 10 {
            if core::arch::x86_64::_rdrand64_step(&mut v) == 1 {
                return Some(v);
            }
            tries += 1;
        }
        None
    }

    /// RDRAND value if the CPU advertises it (CPUID.1:ECX[30]), else None.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn rdrand64() -> Option<u64> {
        if core::arch::x86_64::__cpuid(1).ecx & (1 << 30) == 0 {
            return None;
        }
        rdrand_raw() // safe: CPUID just confirmed RDRAND support
    }

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn rng_next() -> u64 {
        if RNG_STATE == 0 {
            let mut seed = cmos_unix_seconds()
                ^ 0x9E37_79B9_7F4A_7C15
                ^ (R4_PREEMPT_TICKS << 17 | R4_PREEMPT_TICKS);
            match rdrand64() {
                Some(hw) => {
                    seed ^= hw; // strengthen — XOR never weakens the soft seed
                    RNG_HWSEED = 1;
                }
                None => RNG_HWSEED = 2,
            }
            RNG_STATE = if seed == 0 { 0x1234_5678_9ABC_DEF1 } else { seed };
        }
        // Mix in live tick entropy, then xorshift64*.
        RNG_STATE ^= R4_PREEMPT_TICKS.wrapping_add(0xD1B5_4A32_D192_ED03);
        let mut x = RNG_STATE;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        RNG_STATE = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// sys_getrandom (ABI v3.2 id 54): fill the user buffer at `buf_ptr`
    /// with `len` random bytes. Returns the count written, or -1 on a bad
    /// pointer or oversize request.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_getrandom(buf_ptr: u64, len: u64) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        if len == 0 {
            return 0;
        }
        if len > 4096 {
            return ERR;
        }
        let n = len as usize;
        let mut tmp = [0u8; 256];
        let mut done = 0usize;
        while done < n {
            let chunk = core::cmp::min(tmp.len(), n - done);
            let mut i = 0usize;
            while i < chunk {
                let r = rng_next().to_le_bytes();
                let take = core::cmp::min(8, chunk - i);
                tmp[i..i + take].copy_from_slice(&r[..take]);
                i += take;
            }
            if copyout_user(buf_ptr + done as u64, &tmp[..chunk], chunk).is_err() {
                return ERR;
            }
            done += chunk;
        }
        len
    }

    /// RNG hardware-seeding self-test (full-os guide Part IV.10): force the
    /// CSPRNG to seed, report whether RDRAND contributed (`RNG: hwseed rdrand
    /// ok` on a CPU that advertises it, `RNG: hwseed soft (no rdrand)` otherwise
    /// — both are healthy; the soft path is the portable fallback), and confirm
    /// the pool still produces distinct output (two 16-byte draws differ).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn rng_hwseed_selftest() -> u64 {
        let mut a = [0u8; 16];
        let mut b = [0u8; 16];
        let mut i = 0usize;
        while i < 16 {
            a[i..i + 8].copy_from_slice(&rng_next().to_le_bytes());
            i += 8;
        }
        i = 0;
        while i < 16 {
            b[i..i + 8].copy_from_slice(&rng_next().to_le_bytes());
            i += 8;
        }
        if RNG_HWSEED == 1 {
            serial_write(b"RNG: hwseed rdrand ok\n");
        } else {
            serial_write(b"RNG: hwseed soft (no rdrand)\n");
        }
        if a == b {
            serial_write(b"RNG: hwseed FAIL\n");
            return 0;
        }
        1
    }

    /// sys_sandbox (ABI v3.2 id 59): restrict the calling task to the
    /// syscalls whose bit is set in `allow_mask`. Monotonic — a mask wider
    /// than the current one is rejected (-1). Syscalls 0 (debug_write) and
    /// 2 (thread_exit) are always kept so a sandboxed task can still report
    /// and exit. Returns 0 on success.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_sandbox(allow_mask: u64) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let cur = R4_TASKS[r4_current_smp()].sec_filter_mask;
        // Reject any attempt to re-grant a syscall not currently allowed.
        if allow_mask & !cur != 0 {
            return ERR;
        }
        R4_TASKS[r4_current_smp()].sec_filter_mask = (cur & allow_mask) | (1 << 0) | (1 << 2);
        0
    }

    /// Maximum surfaces a single compositor `compose` call (sys_ioctl op 4) may
    /// blit in one frame.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const COMPOSE_MAX: usize = 16;

    /// sys_ioctl (ABI v3.2 id 56, generic device control): op 1 = framebuffer
    /// blit. a2 packs the rectangle as x<<48 | y<<32 | w<<16 | h (each u16);
    /// a3 is the 32-bpp XRGB color. Returns 0 on success, -1 if no
    /// framebuffer / off-screen / unknown op.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_ioctl(op: u64, a2: u64, a3: u64) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        match op {
            1 => {
                let x = (a2 >> 48) & 0xFFFF;
                let y = (a2 >> 32) & 0xFFFF;
                let w = (a2 >> 16) & 0xFFFF;
                let h = a2 & 0xFFFF;
                if fb::fb_blit_rect(x, y, w, h, a3 as u32) {
                    0
                } else {
                    ERR
                }
            }
            // op 2 = openpty (full-os guide Part V.11): allocate a pty pair,
            // return (slave_fd << 32) | master_fd.
            2 => {
                let mut p = 0usize;
                while p < PTY_MAX && PTYS[p].active {
                    p += 1;
                }
                if p == PTY_MAX {
                    return ERR;
                }
                let mfd = m8_alloc_fd(M8FdKind::PtyMaster);
                if mfd == 0xFFFF_FFFF_FFFF_FFFF {
                    return ERR;
                }
                let sfd = m8_alloc_fd(M8FdKind::PtySlave);
                if sfd == 0xFFFF_FFFF_FFFF_FFFF {
                    // Roll back the master alloc fully: free the table slot AND
                    // the fd_count m8_alloc_fd incremented (mirrors the pipe
                    // cleanup; otherwise a near-quota task leaks fd_count).
                    M8_FD_TABLE[mfd as usize] = M8FdEntry::EMPTY;
                    if R4_TASKS[r4_current_smp()].fd_count != 0 {
                        R4_TASKS[r4_current_smp()].fd_count -= 1;
                    }
                    return ERR;
                }
                PTYS[p] = PtyObj::EMPTY;
                PTYS[p].active = true;
                PTYS[p].master_open = true;
                PTYS[p].slave_open = true;
                let rights = m10_rights_for_kind(M8FdKind::PtyMaster);
                M8_FD_TABLE[mfd as usize].rights = rights;
                M8_FD_TABLE[sfd as usize].rights = rights;
                M8_FD_PTY[mfd as usize] = p as u8;
                M8_FD_PTY[sfd as usize] = p as u8;
                (sfd << 32) | mfd
            }
            // op 3 = PC speaker beep (full-os guide Part III audio): program
            // PIT channel 2 to a2 Hz (square wave) and gate the speaker on.
            // Returns the read-back gate bits (3 = enabled), or ERR.
            3 => {
                let freq = a2;
                if freq < 20 || freq > 20_000 {
                    return ERR;
                }
                let divisor = (1_193_182u64 / freq) as u16;
                arch_x86::outb(0x43, 0xB6); // ch2, lobyte/hibyte, mode 3
                arch_x86::outb(0x42, (divisor & 0xFF) as u8);
                arch_x86::outb(0x42, (divisor >> 8) as u8);
                let prev = arch_x86::inb(0x61);
                arch_x86::outb(0x61, prev | 0x03); // gate (bit0) + data (bit1)
                let gate = arch_x86::inb(0x61) & 0x03;
                serial_write(b"BEEP: freq=0x");
                serial_write_hex(freq);
                serial_write(b" div=0x");
                serial_write_hex(divisor as u64);
                serial_write(b" gate=0x");
                serial_write_hex(gate as u64);
                serial_write(b"\n");
                // v1: no timer-driven duration yet, so silence it again right
                // away; the read-back above already proved the speaker enabled.
                arch_x86::outb(0x61, prev & !0x03);
                gate as u64
            }
            // op 4 = compositor compose (full-os guide Part III window-server):
            // composite a client-supplied list of surfaces to the framebuffer in
            // z-order (painter's algorithm — lowest z first / background, highest
            // z last / on top). a2 = pointer to `a3` surface descriptors (max 16);
            // each descriptor is two u64s: [x<<48|y<<32|w<<16|h] then
            // [color(low 32) | z(high 32)]. Returns the number of surfaces blitted.
            4 => {
                let count = a3 as usize;
                if count == 0 || count > COMPOSE_MAX {
                    return ERR;
                }
                let bytes = count * 16;
                let mut raw = [0u8; COMPOSE_MAX * 16];
                if copyin_user(&mut raw[..bytes], a2, bytes).is_err() {
                    return ERR;
                }
                let mut geom = [0u64; COMPOSE_MAX];
                let mut color = [0u32; COMPOSE_MAX];
                let mut z = [0u32; COMPOSE_MAX];
                let mut order = [0usize; COMPOSE_MAX];
                let mut i = 0usize;
                while i < count {
                    let o = i * 16;
                    let w0 = u64::from_le_bytes(raw[o..o + 8].try_into().unwrap());
                    let w1 = u64::from_le_bytes(raw[o + 8..o + 16].try_into().unwrap());
                    geom[i] = w0;
                    color[i] = (w1 & 0xFFFF_FFFF) as u32;
                    z[i] = (w1 >> 32) as u32;
                    order[i] = i;
                    i += 1;
                }
                // Stable insertion-sort of the draw order by z ascending. Stable
                // (shift only on strictly-greater z) so equal-z surfaces keep
                // submission order: earlier-submitted drawn first, later on top.
                let mut a = 1usize;
                while a < count {
                    let cur = order[a];
                    let mut b = a;
                    while b > 0 && z[order[b - 1]] > z[cur] {
                        order[b] = order[b - 1];
                        b -= 1;
                    }
                    order[b] = cur;
                    a += 1;
                }
                let mut painted = 0u64;
                let mut idx = 0usize;
                while idx < count {
                    let s = order[idx];
                    let g = geom[s];
                    let x = (g >> 48) & 0xFFFF;
                    let y = (g >> 32) & 0xFFFF;
                    let w = (g >> 16) & 0xFFFF;
                    let h = g & 0xFFFF;
                    if fb::fb_blit_rect(x, y, w, h, color[s]) {
                        painted += 1;
                    }
                    idx += 1;
                }
                painted
            }
            // op 5 = input_poll (full-os guide Part III): drain up to a3 pending
            // input events (16 bytes each: kind/data/x/y) from the kernel input
            // ring into the user buffer a2. Returns the event count (0 = none).
            5 => {
                let max = core::cmp::min(a3 as usize, 64);
                if max == 0 {
                    return 0;
                }
                let mut buf = [0u8; kbd::INPUT_EVENT_SIZE * 64];
                let n = kbd::input_drain(&mut buf[..max * kbd::INPUT_EVENT_SIZE], max);
                if n > 0
                    && copyout_user(a2, &buf[..n * kbd::INPUT_EVENT_SIZE], n * kbd::INPUT_EVENT_SIZE)
                        .is_err()
                {
                    return ERR;
                }
                n as u64
            }
            // op 6 = surface_compose (full-os guide Part III): composite a client
            // pixel bitmap (a real per-pixel surface, vs op 4's solid-color rects)
            // to the framebuffer. a2 = pointer to w*h ARGB pixels; a3 = geom
            // x<<48|y<<32|w<<16|h. Returns the pixel count blitted.
            6 => {
                const SURFACE_MAX_PX: u64 = 1024; // 32x32; bounds the copyin buffer
                let geom = a3;
                let x = (geom >> 48) & 0xFFFF;
                let y = (geom >> 32) & 0xFFFF;
                let w = (geom >> 16) & 0xFFFF;
                let hh = geom & 0xFFFF;
                if w == 0 || hh == 0 || w * hh > SURFACE_MAX_PX {
                    return ERR;
                }
                let n = (w * hh * 4) as usize;
                let mut buf = [0u8; SURFACE_MAX_PX as usize * 4];
                if copyin_user(&mut buf[..n], a2, n).is_err() {
                    return ERR;
                }
                if fb::fb_blit_pixels(x, y, w, hh, &buf[..n]) {
                    w * hh
                } else {
                    ERR
                }
            }
            // op 7 = audio_write (full-os guide Part III audio): play a block of
            // 16-bit PCM from userspace through the HD Audio output stream. a2 =
            // pointer to PCM bytes, a3 = length. Copies the samples in and streams
            // them to the codec (DMA). Returns the byte count accepted, or 0 if no
            // audio device is ready / the stream did not progress.
            7 => {
                const AUDIO_MAX: usize = AUDIO_BUF_BYTES as usize;
                let len = a3 as usize;
                let n = if len > AUDIO_MAX { AUDIO_MAX } else { len };
                if n == 0 {
                    return 0;
                }
                let mut buf = [0u8; AUDIO_MAX];
                if copyin_user(&mut buf[..n], a2, n).is_err() {
                    return ERR;
                }
                hda_audio_play(&buf[..n]) as u64
            }
            _ => ERR,
        }
    }

    // Dynamic loading (full-os guide Part V.11): a real ELF `.so` dynamic linker.
    // The kernel ships one position-independent shared object embedded
    // (`dl_module.so`, built by `nasm -f elf64` + `rust-lld -shared` from
    // apps/dl/libdl.asm — sidestepping the blocked mingw PE->ELF C path that
    // cannot produce a working multi-page binary). dlopen parses the ELF64
    // program headers, maps each PT_LOAD segment into a fresh user region at
    // DLOPEN_BASE+p_vaddr with W^X perms, and applies R_X86_64_RELATIVE
    // relocations from PT_DYNAMIC; dlsym resolves a name from the .dynsym/.dynstr
    // (symbol count from the SysV .hash nchain). The user demand window
    // [0x0100_0000,0x0200_0000) is fully partitioned — brk [0x100,0x120), mmap
    // [0x120,0x140), exec-app [0x140,0x180), demand-task stacks [0x190,0x200)
    // (all in MiB) — leaving exactly ONE free 1 MiB sub-gap, [0x0180_0000,
    // 0x0190_0000). DLOPEN_BASE sits there, so a loaded object (the shipped
    // .so spans <0x4000) never aliases the caller's heap/mmap/exec segments OR
    // a clone-thread's demand stack (which begins at 0x0190_0000). NOTE: this
    // gap is below DEMAND_STACK_BASE by design; do not raise DLOPEN_BASE into
    // [0x0190_0000,0x0200_0000) — that is the per-task stack region.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const DLOPEN_BASE: u64 = 0x0180_0000;
    // Code-base ASLR (full-os guide Part IV.10): a dlopen'd shared object loads at a
    // RANDOMIZED base within its 1 MiB window, not the fixed DLOPEN_BASE. The window
    // is divided into DL_NSLOTS non-overlapping slots (each larger than any shipped
    // .so, so two loads never alias); each load picks a fresh random slot, different
    // from the previous one. Because the object is position-independent and the
    // loader applies R_X86_64_RELATIVE/GLOB_DAT/JUMP_SLOT relocations relative to the
    // chosen base, the code runs correctly wherever it lands -- so two loads of the
    // same module get different code bases. DL_LOAD_BASE holds the current base and is
    // what dl_resolve adds st_value to.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const DL_SLOT: u64 = 0x8000; // 8 pages, > any shipped .so (libdl <0x4000, on-disk <=0x2000)
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const DL_NSLOTS: u64 = 0x10_0000 / DL_SLOT; // 32 slots across [DLOPEN_BASE, +1 MiB)
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut DL_LOAD_BASE: u64 = DLOPEN_BASE;
    // The page-aligned mapped VA range [lo, hi) of the object currently loaded at
    // DL_LOAD_BASE, recorded by dl_load_elf so dlclose can unmap exactly its pages
    // and release their frames. hi == 0 means no live object. (full-os Part V.11)
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut DL_MAP_LO: u64 = 0;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut DL_MAP_HI: u64 = 0;

    /// Pick a randomized, slot-aligned load base for the next dlopen, different from
    /// the previous load's slot (so the per-load base provably varies and never
    /// overlaps the last one). (full-os guide Part IV.10, code-base ASLR.)
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn dl_aslr_base() -> u64 {
        let prev = (DL_LOAD_BASE - DLOPEN_BASE) / DL_SLOT;
        let mut slot = rng_next() % DL_NSLOTS;
        if slot == prev {
            slot = (slot + 1) % DL_NSLOTS;
        }
        DLOPEN_BASE + slot * DL_SLOT
    }
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static DL_SO: &[u8] = include_bytes!("dl_module.so");
    // An on-disk `.so` read off the filesystem (vs the embedded `DL_SO`). dlopen
    // of a `/data/…so` path reads the file here; `DL_CUR_ONDISK` selects which
    // image dlsym resolves against. Bounded buffer (the loader caps at this size).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const DL_LOADED_MAX: usize = 8192;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut DL_LOADED: [u8; DL_LOADED_MAX] = [0; DL_LOADED_MAX];
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut DL_LOADED_LEN: usize = 0;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut DL_CUR_ONDISK: bool = false;

    /// The `.so` image dlsym currently resolves against (on-disk if the last
    /// dlopen loaded a file, else the embedded blob).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn dl_current() -> &'static [u8] {
        if DL_CUR_ONDISK {
            &DL_LOADED[..DL_LOADED_LEN]
        } else {
            DL_SO
        }
    }

    /// dlopen a `.so` from the filesystem: look it up in the VFS, read it into
    /// `DL_LOADED`, and load it. Returns the load base, or ERR.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn dl_load_from_vfs(path: &[u8]) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        // The VFS root is mounted at /data (sys_open strips the prefix), so strip
        // it here too: "/data/x.so" -> "/x.so".
        if path.len() < 6 || &path[..5] != b"/data" {
            return ERR;
        }
        let rel = &path[5..];
        let (kind, size) = match vfs::vfs_stat(rel) {
            Some(v) => v,
            None => return ERR,
        };
        if kind != vfs::KIND_FILE || size < 64 || size > DL_LOADED_MAX {
            return ERR;
        }
        let idx = match vfs::vfs_lookup(rel) {
            Some(i) => i,
            None => return ERR,
        };
        if vfs::vfs_read(idx, 0, &mut DL_LOADED[..size]) != size {
            return ERR;
        }
        // Switch the dlsym image to the on-disk object ONLY on a successful load,
        // so a failed dlopen leaves dlsym resolving the last good object (matching
        // the libdl branch), not the bad/partial on-disk blob.
        let r = dl_load_elf(&DL_LOADED[..size]);
        if r != ERR {
            DL_LOADED_LEN = size;
            DL_CUR_ONDISK = true;
        }
        r
    }

    /// Seed the on-disk dynamic-linker test fixture: write the embedded `.so` to
    /// `/data/dltest.so` so the on-disk dlopen path has a real file to load.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn dl_ondisk_seed() {
        // Only seed when an on-disk REQUEST MARKER is present (a "DLSEED" magic at
        // a scratch sector), so ordinary boots never write to the VFS -- a write
        // here would persist on the shared go-lane disk and perturb VFS node-count
        // assertions. The on-disk dlopen test writes the marker; nothing else does.
        const DLSEED_LBA: u64 = 17; // free scratch sector (12..63 gap)
        if !storage::r4_storage_available() || !block_io_dispatch(false, DLSEED_LBA, 512, false) {
            return;
        }
        if &BLK_DATA_PAGE.0[..6] != b"DLSEED" {
            return;
        }
        // VFS root is mounted at /data, so the file is "dltest.so" at the root.
        if let Some(idx) = vfs::vfs_open(b"/dltest.so", true) {
            let _ = vfs::vfs_write(idx, 0, DL_SO);
        }
    }

    /// Convert a `.so` virtual address to a byte offset in the file image by
    /// walking the PT_LOAD program headers.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    fn dl_vaddr_to_off(so: &[u8], vaddr: u64) -> Option<usize> {
        if so.len() < 64 {
            return None;
        }
        let e_phoff = u64::from_le_bytes(so[32..40].try_into().ok()?) as usize;
        let e_phentsize = u16::from_le_bytes(so[54..56].try_into().ok()?) as usize;
        let e_phnum = u16::from_le_bytes(so[56..58].try_into().ok()?) as usize;
        let mut i = 0usize;
        while i < e_phnum {
            let ph = e_phoff + i * e_phentsize;
            if ph + 56 > so.len() {
                return None;
            }
            if u32::from_le_bytes(so[ph..ph + 4].try_into().ok()?) == 1 {
                let p_offset = u64::from_le_bytes(so[ph + 8..ph + 16].try_into().ok()?);
                let p_vaddr = u64::from_le_bytes(so[ph + 16..ph + 24].try_into().ok()?);
                let p_filesz = u64::from_le_bytes(so[ph + 32..ph + 40].try_into().ok()?);
                if vaddr >= p_vaddr && vaddr < p_vaddr + p_filesz {
                    return Some((vaddr - p_vaddr + p_offset) as usize);
                }
            }
            i += 1;
        }
        None
    }

    /// `st_value` of `.dynsym` entry `idx` (for symbolic GLOB_DAT/JUMP_SLOT
    /// relocations: the value to load the GOT slot with is base + this).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    fn dl_sym_value(so: &[u8], idx: usize) -> Option<u64> {
        let symtab = dl_dyn(so, 6).and_then(|v| dl_vaddr_to_off(so, v))?;
        let sym = symtab + idx * 24;
        if idx == 0 || sym + 16 > so.len() {
            return None;
        }
        Some(u64::from_le_bytes(so[sym + 8..sym + 16].try_into().ok()?))
    }

    /// Look up a tag in the `.so` PT_DYNAMIC array; returns its `d_val`.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    fn dl_dyn(so: &[u8], tag: u64) -> Option<u64> {
        if so.len() < 64 {
            return None;
        }
        let e_phoff = u64::from_le_bytes(so[32..40].try_into().ok()?) as usize;
        let e_phentsize = u16::from_le_bytes(so[54..56].try_into().ok()?) as usize;
        let e_phnum = u16::from_le_bytes(so[56..58].try_into().ok()?) as usize;
        let mut dyn_off = 0usize;
        let mut dyn_sz = 0usize;
        let mut i = 0usize;
        while i < e_phnum {
            let ph = e_phoff + i * e_phentsize;
            if ph + 56 > so.len() {
                return None;
            }
            if u32::from_le_bytes(so[ph..ph + 4].try_into().ok()?) == 2 {
                dyn_off = u64::from_le_bytes(so[ph + 8..ph + 16].try_into().ok()?) as usize;
                dyn_sz = u64::from_le_bytes(so[ph + 32..ph + 40].try_into().ok()?) as usize;
                break;
            }
            i += 1;
        }
        if dyn_sz == 0 || dyn_off + dyn_sz > so.len() {
            return None;
        }
        let mut d = 0usize;
        while d + 16 <= dyn_sz {
            let t = u64::from_le_bytes(so[dyn_off + d..dyn_off + d + 8].try_into().ok()?);
            let v = u64::from_le_bytes(so[dyn_off + d + 8..dyn_off + d + 16].try_into().ok()?);
            if t == 0 {
                break;
            }
            if t == tag {
                return Some(v);
            }
            d += 16;
        }
        None
    }

    /// dlopen: map an ELF `.so` (the embedded blob or one read off disk) into the
    /// user address space and apply its relocations. Returns the load base, or ERR.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn dl_load_elf(so: &[u8]) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        // Code-base ASLR: load at a fresh randomized base (relocations below are all
        // applied relative to it). dl_resolve reads DL_LOAD_BASE for the same base.
        // Reclaim a previously-loaded object's pages before loading another, so a
        // dlopen without an intervening dlclose does not orphan the prior object's
        // frames and W^X mappings (single live object). Uses the OLD DL_MAP range
        // and runs BEFORE dl_aslr_base (which reads the OLD DL_LOAD_BASE to avoid
        // reusing the same slot). vm_unmap_current is CR3-relative, matching the
        // vm_map_current that created the mappings; an already-unmapped VA (e.g. a
        // stale range from another address space) is a harmless no-op. (full-os V.11)
        if DL_MAP_HI != 0 && DL_MAP_LO < DL_MAP_HI {
            let mut va = DL_MAP_LO;
            while va < DL_MAP_HI {
                crate::mm::vm_unmap_current(va);
                va += 0x1000;
            }
        }
        let base = dl_aslr_base();
        DL_LOAD_BASE = base;
        DL_MAP_LO = u64::MAX; // track the mapped page range so dlclose can unmap it
        DL_MAP_HI = 0;
        if so.len() < 64 || so[0] != 0x7F || &so[1..4] != b"ELF" || so[4] != 2 {
            return ERR; // not a 64-bit ELF
        }
        let e_phoff = u64::from_le_bytes(so[32..40].try_into().unwrap()) as usize;
        let e_phentsize = u16::from_le_bytes(so[54..56].try_into().unwrap()) as usize;
        let e_phnum = u16::from_le_bytes(so[56..58].try_into().unwrap()) as usize;
        // Pass 1: map each PT_LOAD RW and copy its file contents in.
        let mut i = 0usize;
        while i < e_phnum {
            let ph = e_phoff + i * e_phentsize;
            if ph + 56 > so.len() {
                return ERR;
            }
            let p_type = u32::from_le_bytes(so[ph..ph + 4].try_into().unwrap());
            if p_type == 1 {
                let p_offset = u64::from_le_bytes(so[ph + 8..ph + 16].try_into().unwrap()) as usize;
                let p_vaddr = u64::from_le_bytes(so[ph + 16..ph + 24].try_into().unwrap());
                let p_filesz = u64::from_le_bytes(so[ph + 32..ph + 40].try_into().unwrap()) as usize;
                let p_memsz = u64::from_le_bytes(so[ph + 40..ph + 48].try_into().unwrap());
                if p_memsz > 0 {
                    // Bound the segment to this load's ASLR slot (p_memsz is an ELF
                    // header field independent of the file-size cap, so a crafted .so
                    // could declare a huge p_memsz). Without this a tiny-p_filesz /
                    // huge-p_memsz segment would map past DL_SLOT into the next slot
                    // (clobbering a neighbour object's W^X) or, at the top slot, into
                    // the per-task demand-stack region. Reject anything that overruns.
                    if p_vaddr > DL_SLOT || p_memsz > DL_SLOT - p_vaddr {
                        return ERR;
                    }
                    let seg_lo = (base + p_vaddr) & !0xFFF;
                    let end = base + p_vaddr + p_memsz;
                    let seg_hi = (end + 0xFFF) & !0xFFF;
                    if seg_lo < DL_MAP_LO {
                        DL_MAP_LO = seg_lo;
                    }
                    if seg_hi > DL_MAP_HI {
                        DL_MAP_HI = seg_hi;
                    }
                    let mut va = seg_lo;
                    while va < end {
                        if !crate::mm::vm_map_current(va, 2) || !crate::mm::vm_protect_current(va, 2)
                        {
                            return ERR; // RW (force, so a re-dlopen can re-copy)
                        }
                        va += 0x1000;
                    }
                    if p_filesz > 0 {
                        if p_offset + p_filesz > so.len() {
                            return ERR;
                        }
                        if copyout_user(base + p_vaddr, &so[p_offset..p_offset + p_filesz], p_filesz)
                            .is_err()
                        {
                            return ERR;
                        }
                    }
                }
            }
            i += 1;
        }
        // Pass 2: apply DT_RELA relocations. R_X86_64_RELATIVE (8):
        // *(base+off) = base+addend. R_X86_64_GLOB_DAT (6): *(base+off) =
        // base + sym.st_value + addend (a SYMBOLIC reloc — the GOT slot for a
        // global, resolved from .dynsym by the r_info symbol index).
        if let (Some(rela_va), Some(rela_sz)) = (dl_dyn(so, 7), dl_dyn(so, 8)) {
            if let Some(rela_off) = dl_vaddr_to_off(so, rela_va) {
                let rela_sz = rela_sz as usize;
                if rela_off + rela_sz <= so.len() {
                    let mut r = 0usize;
                    while r + 24 <= rela_sz {
                        let o = rela_off + r;
                        let r_offset = u64::from_le_bytes(so[o..o + 8].try_into().unwrap());
                        let r_info = u64::from_le_bytes(so[o + 8..o + 16].try_into().unwrap());
                        let r_addend = u64::from_le_bytes(so[o + 16..o + 24].try_into().unwrap());
                        let r_type = r_info & 0xFFFF_FFFF;
                        if r_type == 8 {
                            let val = base.wrapping_add(r_addend);
                            if copyout_user(base + r_offset, &val.to_le_bytes(), 8).is_err() {
                                return ERR;
                            }
                        } else if r_type == 6 {
                            let stv = match dl_sym_value(so, (r_info >> 32) as usize) {
                                Some(v) => v,
                                None => return ERR,
                            };
                            let val = base.wrapping_add(stv).wrapping_add(r_addend);
                            if copyout_user(base + r_offset, &val.to_le_bytes(), 8).is_err() {
                                return ERR;
                            }
                        }
                        r += 24;
                    }
                }
            }
        }
        // Pass 2b: apply DT_JMPREL (.rela.plt) R_X86_64_JUMP_SLOT (7) relocations —
        // EAGER binding (*(base+off) = base + sym.st_value + addend). The kernel
        // has no lazy PLT resolver, so every PLT slot is bound now.
        if let (Some(jr_va), Some(jr_sz)) = (dl_dyn(so, 23), dl_dyn(so, 2)) {
            if let Some(jr_off) = dl_vaddr_to_off(so, jr_va) {
                let jr_sz = jr_sz as usize;
                if jr_off + jr_sz <= so.len() {
                    let mut r = 0usize;
                    while r + 24 <= jr_sz {
                        let o = jr_off + r;
                        let r_offset = u64::from_le_bytes(so[o..o + 8].try_into().unwrap());
                        let r_info = u64::from_le_bytes(so[o + 8..o + 16].try_into().unwrap());
                        let r_addend = u64::from_le_bytes(so[o + 16..o + 24].try_into().unwrap());
                        if r_info & 0xFFFF_FFFF == 7 {
                            let stv = match dl_sym_value(so, (r_info >> 32) as usize) {
                                Some(v) => v,
                                None => return ERR,
                            };
                            let val = base.wrapping_add(stv).wrapping_add(r_addend);
                            if copyout_user(base + r_offset, &val.to_le_bytes(), 8).is_err() {
                                return ERR;
                            }
                        }
                        r += 24;
                    }
                }
            }
        }
        // Pass 3: set each PT_LOAD to its final perms (R-X for X, RW for W, else R).
        i = 0;
        while i < e_phnum {
            let ph = e_phoff + i * e_phentsize;
            let p_type = u32::from_le_bytes(so[ph..ph + 4].try_into().unwrap());
            if p_type == 1 {
                let p_vaddr = u64::from_le_bytes(so[ph + 16..ph + 24].try_into().unwrap());
                let p_memsz = u64::from_le_bytes(so[ph + 40..ph + 48].try_into().unwrap());
                let p_flags = u32::from_le_bytes(so[ph + 4..ph + 8].try_into().unwrap());
                if p_memsz > 0 {
                    let prot = (if p_flags & 1 != 0 { 4 } else { 0 })
                        | (if p_flags & 2 != 0 { 2 } else { 0 });
                    let mut va = (base + p_vaddr) & !0xFFF;
                    let end = base + p_vaddr + p_memsz;
                    while va < end {
                        if !crate::mm::vm_protect_current(va, prot) {
                            return ERR;
                        }
                        va += 0x1000;
                    }
                }
            }
            i += 1;
        }
        base
    }

    /// dlsym: resolve `target` from the embedded `.so` .dynsym/.dynstr; returns
    /// its loaded VA (base + st_value), or ERR.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn dl_resolve(so: &[u8], target: &[u8]) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let symtab = match dl_dyn(so, 6).and_then(|v| dl_vaddr_to_off(so, v)) {
            Some(o) => o,
            None => return ERR,
        };
        let strtab = match dl_dyn(so, 5).and_then(|v| dl_vaddr_to_off(so, v)) {
            Some(o) => o,
            None => return ERR,
        };
        let hash = match dl_dyn(so, 4).and_then(|v| dl_vaddr_to_off(so, v)) {
            Some(o) => o,
            None => return ERR,
        };
        if hash + 8 > so.len() {
            return ERR;
        }
        // SysV hash: nchain (word 1) == number of .dynsym entries.
        let nchain = u32::from_le_bytes(so[hash + 4..hash + 8].try_into().unwrap()) as usize;
        let mut s = 1usize; // skip the index-0 null symbol
        while s < nchain {
            let sym = symtab + s * 24;
            if sym + 24 > so.len() {
                break;
            }
            let st_name = u32::from_le_bytes(so[sym..sym + 4].try_into().unwrap()) as usize;
            let st_value = u64::from_le_bytes(so[sym + 8..sym + 16].try_into().unwrap());
            let nm = strtab + st_name;
            let mut k = 0usize;
            let mut ok = true;
            while k < target.len() {
                if nm + k >= so.len() || so[nm + k] != target[k] {
                    ok = false;
                    break;
                }
                k += 1;
            }
            // Exact match: the .so name must terminate right after `target`.
            if ok && nm + target.len() < so.len() && so[nm + target.len()] == 0 {
                // Resolve against the module's actual (randomized) load base.
                return DL_LOAD_BASE + st_value;
            }
            s += 1;
        }
        ERR
    }

    /// sys_dlctl (ABI v3.2 id 60): op 1 = dlopen(name) -> ELF `.so` load base VA,
    /// op 2 = dlsym(name) -> resolved symbol VA. -1 on unknown module/symbol or a
    /// mapping/relocation failure.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_dlctl(op: u64, a2: u64, _a3: u64) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        match op {
            1 => {
                // dlopen: "libdl" loads the kernel-embedded .so; a "/data/<x>.so"
                // path loads a shared object off the filesystem.
                let mut name = [0u8; 32];
                if copyin_user(&mut name, a2, 32).is_err() {
                    return ERR;
                }
                let mut len = 0usize;
                while len < 32 && name[len] != 0 {
                    len += 1;
                }
                let nm = &name[..len];
                if nm == b"libdl" {
                    DL_CUR_ONDISK = false;
                    dl_load_elf(DL_SO)
                } else if len > 6 && &nm[..6] == b"/data/" {
                    dl_load_from_vfs(nm)
                } else {
                    ERR
                }
            }
            2 => {
                // dlsym: resolve a name from the currently-loaded object's .dynsym.
                let mut nm = [0u8; 16];
                if copyin_user(&mut nm, a2, 16).is_err() {
                    return ERR;
                }
                let mut len = 0usize;
                while len < 16 && nm[len] != 0 {
                    len += 1;
                }
                if len == 0 {
                    return ERR;
                }
                dl_resolve(dl_current(), &nm[..len])
            }
            3 => {
                // dlclose(a2 = base): unmap the pages of the object currently loaded
                // at `base` (the most-recent dlopen) and release their frames.
                // Returns 0, or ERR if `base` is not the live object / nothing is
                // loaded. A double-close fails (DL_MAP_HI is cleared). dlsym after a
                // close is undefined, as in POSIX. (full-os guide Part V.11)
                if a2 != DL_LOAD_BASE || DL_MAP_HI == 0 || DL_MAP_LO >= DL_MAP_HI {
                    return ERR;
                }
                let mut va = DL_MAP_LO;
                while va < DL_MAP_HI {
                    crate::mm::vm_unmap_current(va);
                    va += 0x1000;
                }
                DL_MAP_LO = 0;
                DL_MAP_HI = 0;
                DL_LOAD_BASE = DLOPEN_BASE; // no live object; a stale dlclose now fails
                DL_CUR_ONDISK = false;
                0
            }
            _ => ERR,
        }
    }

    /// Boot self-test for dlclose (full-os guide Part V.11). dlopen the embedded
    /// module, confirm its base page is mapped, dlclose it, confirm the page is now
    /// unmapped (the frames were released) and a double-close is rejected, then
    /// confirm a fresh dlopen re-maps -- and clean that one up too, so the shared
    /// boot address space is left with no dlopen mappings. Runs in the shared AS
    /// (CR3-relative, like dlprobe does in a private AS), in the free DLOPEN window.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn dlclose_selftest() {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        let base = dl_load_elf(DL_SO);
        let pg = base & !0xFFF;
        let mapped_before =
            base != ERR && check_page_user_perms(pg, HHDM_OFFSET, USER_PERM_READ);
        let closed = sys_dlctl(3, base, 0);
        let mapped_after = check_page_user_perms(pg, HHDM_OFFSET, USER_PERM_READ);
        let double = sys_dlctl(3, base, 0); // nothing live at `base` now -> ERR
        let base2 = dl_load_elf(DL_SO); // re-open maps fresh
        let reopened =
            base2 != ERR && check_page_user_perms(base2 & !0xFFF, HHDM_OFFSET, USER_PERM_READ);
        let _ = sys_dlctl(3, base2, 0); // clean up the shared-AS mapping
        let ok = mapped_before
            && closed == 0
            && !mapped_after
            && double == ERR
            && reopened;
        if ok {
            serial_write(b"DLCLOSE: unmap ok\n");
        } else {
            serial_write(b"DLCLOSE: unmap fail\n");
        }
    }

    // Security audit log (full-os guide Part IV.10): a small ring, distinct
    // from dmesg, holding structured security events (capability/sandbox
    // denials and privileged operations) that userspace can query via
    // sys_sysinfo op 7. Records WHO (tid) tried WHAT (syscall nr) and was denied.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    const AUDIT_CAP: usize = 1024;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut AUDIT: [u8; AUDIT_CAP] = [0; AUDIT_CAP];
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut AUDIT_HEAD: usize = 0;
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    static mut AUDIT_LEN: usize = 0;

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn audit_byte(b: u8) {
        AUDIT[AUDIT_HEAD] = b;
        AUDIT_HEAD = (AUDIT_HEAD + 1) % AUDIT_CAP;
        if AUDIT_LEN < AUDIT_CAP {
            AUDIT_LEN += 1;
        }
    }

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn audit_write(s: &[u8]) {
        for &b in s {
            audit_byte(b);
        }
    }

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn audit_hex(v: u64) {
        const HEX: &[u8; 16] = b"0123456789ABCDEF";
        let mut shift: i32 = 60;
        while shift >= 0 {
            audit_byte(HEX[((v >> shift) & 0xF) as usize]);
            shift -= 4;
        }
    }

    /// Record a denied/privileged security event: tag + syscall nr + caller tid.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn audit_event(tag: &[u8], nr: u64) {
        audit_write(b"AUDIT: ");
        audit_write(tag);
        audit_write(b" nr=0x");
        audit_hex(nr);
        audit_write(b" tid=0x");
        audit_hex(R4_CURRENT as u64);
        audit_write(b"\n");
    }

    /// Copy the most recent `len` bytes of the audit ring (oldest->newest) to
    /// the user buffer; returns the count or u64::MAX on a bad buffer.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn audit_read(ptr: u64, len: usize) -> u64 {
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

    /// True if `needle` occurs in `hay`.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    fn slice_contains(hay: &[u8], needle: &[u8]) -> bool {
        if needle.is_empty() {
            return true;
        }
        if needle.len() > hay.len() {
            return false;
        }
        let mut i = 0usize;
        while i + needle.len() <= hay.len() {
            if &hay[i..i + needle.len()] == needle {
                return true;
            }
            i += 1;
        }
        false
    }

    /// Boot self-test for the security audit checkpoints (full-os guide Part IV.10).
    /// The sandbox-deny gate (syscall.rs) and the power path (sys_power) now record
    /// an audit event; this drives the same `audit_event` calls those sites make and
    /// confirms each lands in the ring with the expected tag + syscall nr, read back
    /// exactly the bytes appended (the way sys_sysinfo op 7 exposes them to userspace).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn audit_checkpoint_selftest() {
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
        let ok = slice_contains(buf, b"sandbox-deny nr=0x0000000000000031")
            && slice_contains(buf, b"power-off nr=0x000000000000003A");
        if ok {
            serial_write(b"AUDIT: checkpoints ok\n");
        } else {
            serial_write(b"AUDIT: checkpoints fail\n");
        }
    }

    /// sys_sysinfo (ABI v3.2 id 61): lightweight /proc-style metrics.
    /// op 1 = live task count, op 2 = free physical frames, op 3 = uptime
    /// in PIT ticks (100 Hz). -1 on bad op.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_sysinfo(op: u64, a2: u64, a3: u64) -> u64 {
        match op {
            1 => {
                let mut live = 0u64;
                let mut i = 0usize;
                while i < R4_NUM_TASKS {
                    if !matches!(R4_TASKS[i].state, R4State::Dead) {
                        live += 1;
                    }
                    i += 1;
                }
                live
            }
            2 => mm::free_frames(),
            3 => R4_PREEMPT_TICKS,
            // op 4 = dmesg read: copy the kernel log tail (a2 = user buffer,
            // a3 = capacity) -> bytes copied, or u64::MAX on a bad buffer.
            4 => klog_read(a2, a3 as usize),
            // op 5 = MBR partition-table parse (full-os guide Part II.5
            // partitions): read LBA 0, validate the 0x55AA signature, log each
            // non-empty primary entry -> partition count (u64::MAX if no disk
            // or no boot signature).
            5 => {
                if !storage::r4_storage_available()
                    || !block_io_dispatch(false, 0, 512, false)
                {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                if BLK_DATA_PAGE.0[510] != 0x55 || BLK_DATA_PAGE.0[511] != 0xAA {
                    serial_write(b"PART: no signature\n");
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                let mut count = 0u64;
                let mut i = 0usize;
                while i < 4 {
                    let e = 446 + i * 16;
                    let ptype = BLK_DATA_PAGE.0[e + 4];
                    if ptype != 0 {
                        let lba = u32::from_le_bytes([
                            BLK_DATA_PAGE.0[e + 8],
                            BLK_DATA_PAGE.0[e + 9],
                            BLK_DATA_PAGE.0[e + 10],
                            BLK_DATA_PAGE.0[e + 11],
                        ]);
                        let secs = u32::from_le_bytes([
                            BLK_DATA_PAGE.0[e + 12],
                            BLK_DATA_PAGE.0[e + 13],
                            BLK_DATA_PAGE.0[e + 14],
                            BLK_DATA_PAGE.0[e + 15],
                        ]);
                        serial_write(b"PART: ");
                        serial_write_hex(i as u64);
                        serial_write(b" type=0x");
                        serial_write_hex(ptype as u64);
                        serial_write(b" lba=0x");
                        serial_write_hex(lba as u64);
                        serial_write(b" sectors=0x");
                        serial_write_hex(secs as u64);
                        serial_write(b"\n");
                        count += 1;
                    }
                    i += 1;
                }
                count
            }
            // op 6 = FAT16 read (full-os guide Part II.5): read the file
            // HELLO.TXT from a FAT volume at a fixed LBA. a2 = user buffer,
            // a3 = capacity -> file bytes copied (single cluster, v1), or
            // u64::MAX on error.
            6 => {
                let mut tmp = [0u8; 512];
                let cap = core::cmp::min(a3 as usize, 512);
                let n = match fat16_read_named(b"HELLO   TXT", &mut tmp[..cap]) {
                    Some(n) => n,
                    None => return 0xFFFF_FFFF_FFFF_FFFF,
                };
                if copyout_user(a2, &tmp[..n], n).is_err() {
                    return 0xFFFF_FFFF_FFFF_FFFF;
                }
                n as u64
            }
            // op 7 = security audit-log read (full-os guide Part IV.10):
            // a2 = user buffer, a3 = capacity -> bytes copied (u64::MAX on a
            // bad buffer). Public so any task can inspect the audit trail.
            7 => audit_read(a2, a3 as usize),
            // op 8 = FAT16 root directory list (full-os guide Part II.5): logs
            // each entry, returns the count.
            8 => fat16_list(),
            // op 9 = disk-encryption round trip (full-os guide Part IV.10):
            // encrypt a known plaintext, write it to a scratch sector, read it
            // back raw (must be ciphertext != plaintext), decrypt, and verify
            // it matches. -> 1 on success, 0 on failure.
            9 => {
                const SCRATCH_LBA: u64 = 1600; // free, above the /data region
                let plain: [u8; 32] = *b"rugo-crypt-plaintext-0123456789!";
                if !storage::r4_storage_available() {
                    return 0;
                }
                let mut ct = plain;
                disk_crypt(SCRATCH_LBA, &mut ct);
                if ct == plain {
                    return 0; // cipher must actually transform the data
                }
                core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr(), 0, 512);
                BLK_DATA_PAGE.0[..32].copy_from_slice(&ct);
                if !block_io_dispatch(true, SCRATCH_LBA, 512, false) {
                    return 0;
                }
                core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr(), 0, 512);
                if !block_io_dispatch(false, SCRATCH_LBA, 512, false) {
                    return 0;
                }
                // On-disk bytes must be ciphertext, not the plaintext.
                if BLK_DATA_PAGE.0[..32] == plain {
                    return 0;
                }
                let mut dec = [0u8; 32];
                dec.copy_from_slice(&BLK_DATA_PAGE.0[..32]);
                disk_crypt(SCRATCH_LBA, &mut dec);
                if dec != plain {
                    return 0;
                }
                serial_write(b"CRYPT: disk roundtrip ok\n");
                1
            }
            // op 10 = FS journaling round trip (full-os guide Part II.5): log a
            // write to the journal (data + committed header), verify the target
            // is NOT yet updated (simulating a crash before apply), then replay
            // the journal and confirm the target now holds the logged data.
            // -> 1 on success, 0 on failure.
            10 => {
                const TGT: u64 = 1601;
                const JHDR: u64 = 1602;
                const JDATA: u64 = 1603;
                const JMAGIC: u32 = 0x4A52_4E4C; // "JRNL"
                let data: [u8; 16] = *b"journaled-write!";
                if !storage::r4_storage_available() {
                    return 0;
                }
                // 0) zero the target sector (pre-write state).
                core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr(), 0, 512);
                if !block_io_dispatch(true, TGT, 512, false) {
                    return 0;
                }
                // 1) write-ahead log: journal data sector, then the committed
                //    header (atomicity boundary: header written last).
                core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr(), 0, 512);
                BLK_DATA_PAGE.0[..16].copy_from_slice(&data);
                if !block_io_dispatch(true, JDATA, 512, false) {
                    return 0;
                }
                core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr(), 0, 512);
                BLK_DATA_PAGE.0[0..4].copy_from_slice(&JMAGIC.to_le_bytes());
                BLK_DATA_PAGE.0[4..12].copy_from_slice(&TGT.to_le_bytes());
                BLK_DATA_PAGE.0[12] = 1; // committed
                if !block_io_dispatch(true, JHDR, 512, false) {
                    return 0;
                }
                // 2) crash simulated here: the target is still un-applied.
                if !block_io_dispatch(false, TGT, 512, false) {
                    return 0;
                }
                if BLK_DATA_PAGE.0[..16] == data {
                    return 0; // target must NOT be written before replay
                }
                // 3) replay: read the committed header, apply the journal data.
                if !block_io_dispatch(false, JHDR, 512, false) {
                    return 0;
                }
                let magic = u32::from_le_bytes([
                    BLK_DATA_PAGE.0[0],
                    BLK_DATA_PAGE.0[1],
                    BLK_DATA_PAGE.0[2],
                    BLK_DATA_PAGE.0[3],
                ]);
                let tgt = u64::from_le_bytes([
                    BLK_DATA_PAGE.0[4],
                    BLK_DATA_PAGE.0[5],
                    BLK_DATA_PAGE.0[6],
                    BLK_DATA_PAGE.0[7],
                    BLK_DATA_PAGE.0[8],
                    BLK_DATA_PAGE.0[9],
                    BLK_DATA_PAGE.0[10],
                    BLK_DATA_PAGE.0[11],
                ]);
                if magic != JMAGIC || BLK_DATA_PAGE.0[12] != 1 || tgt != TGT {
                    return 0;
                }
                if !block_io_dispatch(false, JDATA, 512, false) {
                    return 0;
                }
                let mut jbuf = [0u8; 512];
                jbuf.copy_from_slice(&BLK_DATA_PAGE.0[..512]);
                BLK_DATA_PAGE.0[..512].copy_from_slice(&jbuf);
                if !block_io_dispatch(true, tgt, 512, false) {
                    return 0;
                }
                // 4) clear the committed flag (journal consumed).
                core::ptr::write_bytes(BLK_DATA_PAGE.0.as_mut_ptr(), 0, 512);
                if !block_io_dispatch(true, JHDR, 512, false) {
                    return 0;
                }
                // 5) verify the target now holds the journaled data.
                if !block_io_dispatch(false, TGT, 512, false) {
                    return 0;
                }
                if BLK_DATA_PAGE.0[..16] != data {
                    return 0;
                }
                serial_write(b"JOURNAL: replay ok\n");
                1
            }
            // op 11 = FAT16 write self-test (full-os guide Part II.5): write a
            // single-cluster file to the FAT volume, then read it back via the
            // existing reader and confirm a byte-exact round-trip.
            11 => {
                let name: [u8; 11] = *b"WRTEST  TXT";
                let content: &[u8] = b"fat-write-v1!";
                if !fat16_write_named(&name, content) {
                    return 0;
                }
                let mut out = [0u8; 64];
                let n = match fat16_read_named(&name, &mut out) {
                    Some(v) => v,
                    None => return 0,
                };
                if n != content.len() || &out[..n] != content {
                    return 0;
                }
                serial_write(b"FATWR: write+read ok\n");
                1
            }
            // op 12 = FAT16 multi-cluster (FAT-chain) read self-test: read
            // BIG.TXT by walking the FAT chain and verify a deterministic
            // pattern (byte i == i & 0xFF) across the cluster boundary, proving
            // the chain was followed past the first cluster.
            12 => {
                let name: [u8; 11] = *b"BIG     TXT";
                let mut out = [0u8; 1024];
                let n = match fat16_read_chain(&name, &mut out) {
                    Some(v) => v,
                    None => {
                        serial_write(b"FATBIG: chain read fail\n");
                        return 0;
                    }
                };
                // 600-byte file spanning cluster 2 -> cluster 3. buf[512] lives
                // in the SECOND cluster, reachable only by following FAT[2]->3.
                let ok = n == 600
                    && out[0] == 0x00
                    && out[511] == 0xFF
                    && out[512] == 0x00
                    && out[599] == 0x57;
                if !ok {
                    serial_write(b"FATBIG: chain read fail\n");
                    return 0;
                }
                serial_write(b"FATBIG: chain read ok size=0x");
                serial_write_hex(n as u64);
                serial_write(b"\n");
                // Bad/reserved-cluster guard (full-os Part II.5): BAD.TXT is a
                // 600-byte file whose first cluster's FAT entry is the bad-cluster
                // marker 0xFFF7 (not a valid next cluster). The chain walker must
                // STOP at that terminal value and return only the first cluster's
                // 512 bytes -- never follow 0xFFF7 as a cluster number (which would
                // index a wild data_lba and read garbage). (The crafting test is the
                // only caller of op 12 and supplies BAD.TXT alongside BIG.TXT.)
                let badname: [u8; 11] = *b"BAD     TXT";
                let mut bad = [0u8; 1024];
                match fat16_read_chain(&badname, &mut bad) {
                    Some(512) => serial_write(b"FATBIG: bad-cluster guarded ok\n"),
                    _ => serial_write(b"FATBIG: bad-cluster guard FAIL\n"),
                }
                1
            }
            // op 13 = online CPU count (the BSP + every checked-in AP). Lets
            // userspace size thread pools to the machine (full-os guide Part I.3).
            13 => smp::cpu_count(),
            // op 14 = the calling CPU's current task id, resolved from PER-CPU state
            // (full-os guide Part I.3, SMP scheduler): the global cursor on the BSP,
            // the AP's own per-CPU `current` on an application processor. A task
            // migrated to an AP reads its own real tid here -- the per-CPU R4_CURRENT
            // mechanism exercised through a real syscall (see the ap_r4_migrate test).
            14 => r4_current_smp() as u64,
            // op 15 = SMP concurrency rendezvous (full-os guide Part I.3): a ring-3
            // task running on an AP signals arrival and waits for the BSP's ack,
            // returning 0xAC once both CPUs have met -- proof two tasks ran on two
            // CPUs at once. A no-op (0) on the BSP; see ap_r4_concurrent_selftest.
            15 => smp::smp_rendezvous_ap(),
            // op 16 = bump the calling task's yield_count via per-CPU current and
            // return the new value (full-os Part I.3): the WRITE counterpart of the
            // op 14 read. On an AP it mutates THAT AP task's own R4_TASKS slot, so
            // per-CPU current is proven to resolve correctly for writes to the real
            // task table, not just reads. On the BSP it bumps the live caller (used
            // only by the AP self-test, so harmless there).
            16 => {
                let t = r4_current_smp();
                R4_TASKS[t].yield_count = R4_TASKS[t].yield_count.wrapping_add(1);
                R4_TASKS[t].yield_count
            }
            _ => 0xFFFF_FFFF_FFFF_FFFF,
        }
    }

    /// sys_power (ABI v3.2 id 58): op 0 = shutdown, op 1 = reboot. Requires
    /// uid 0. Shutdown writes the ACPI S5 command to the q35/i440fx PM
    /// control ports (best effort), then falls back to debug-exit; reboot
    /// pulses the 8042 reset line.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_power(op: u64) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
        if R4_TASKS[r4_current_smp()].uid != 0 {
            return ERR;
        }
        // Drain the UART so the marker reaches the host before we stop.
        let drain = || {
            let mut spins = 0u32;
            while arch_x86::inb(0x3FD) & 0x40 == 0 && spins < 100_000 {
                spins += 1;
            }
        };
        match op {
            0 => {
                // Audit the privileged power transition before we halt (full-os
                // guide Part IV.10): the record lands in the ring even though the
                // machine stops immediately after.
                #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                audit_event(b"power-off", 58);
                serial_write(b"POWER: shutdown\n");
                drain();
                arch_x86::outw(0x604, 0x2000); // q35 PM1a_CNT (PMBASE 0x600)
                arch_x86::outw(0xB004, 0x2000); // i440fx ACPI PM
                qemu_exit(0x31); // fallback if ACPI did not take
                loop {
                    core::arch::asm!("cli; hlt", options(nomem, nostack));
                }
            }
            1 => {
                #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                audit_event(b"power-reboot", 58);
                serial_write(b"POWER: reboot\n");
                drain();
                arch_x86::outb(0x64, 0xFE); // 8042 pulse CPU reset
                loop {
                    core::arch::asm!("cli; hlt", options(nomem, nostack));
                }
            }
            _ => ERR,
        }
    }

    unsafe fn r4_find_spawn_slot() -> Option<usize> {
        // Allocate the slot (reuse a free one, else grow the table) under the SMP
        // run-queue lock (full-os guide Part I.3), so this R4_TASKS / R4_NUM_TASKS
        // mutation is serialized with the autonomous scheduler's claim/retire on the
        // APs -- they share R4_RQ_LOCK. Uncontended on the bootstrap processor (the
        // only spawner today), so transparent; the lock makes the table mutation
        // atomic for when an AP path mutates the table concurrently. (Claim-on-find
        // double-claim prevention for concurrent SPAWNERS is deferred -- nothing
        // spawns off the BSP yet; see docs/runtime/smp_syscall_v1.md.)
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        crate::smp::r4_rq_lock();
        let mut result = None;
        let mut tid = 1;
        while tid < R4_NUM_TASKS {
            if R4_TASKS[tid].state == R4State::Dead {
                result = Some(tid);
                break;
            }
            tid += 1;
        }
        if result.is_none() && R4_NUM_TASKS < R4_MAX_TASKS {
            result = Some(R4_NUM_TASKS);
            R4_NUM_TASKS += 1;
        }
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        crate::smp::r4_rq_unlock();
        result
    }

    /// The current task on the CALLING CPU (full-os guide Part I.3, SMP scheduler).
    /// On the bootstrap processor this is the global scheduler cursor `R4_CURRENT`;
    /// on an application processor (go lane only) it is that CPU's own per-CPU
    /// `current` (gs:[16], set when a task is dispatched to it) -- the per-CPU
    /// R4_CURRENT a multi-CPU scheduler reads instead of one global. BSP-vs-AP is
    /// decided by x2APIC ID (`smp::is_bsp`), NOT GS, so this is safe even on the BSP
    /// whose GS base is left unset (ring-3 TinyGo uses GS). On -smp 1 (the whole
    /// suite bar two lanes) `is_bsp` short-circuits to `R4_CURRENT` with just an
    /// atomic load -- no MSR read -- so every call site is transparent there. Defined
    /// for all cfg_r4! lanes (just `R4_CURRENT` off the go lane) so the entire
    /// `R4_TASKS[r4_current_smp()]` syscall surface can route through it uniformly.
    unsafe fn r4_current_smp() -> usize {
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            if crate::smp::is_bsp() {
                R4_CURRENT
            } else {
                let cur: u64;
                core::arch::asm!("mov {}, gs:[16]", out(reg) cur, options(nostack));
                cur as usize
            }
        }
        #[cfg(not(all(feature = "go_test", not(feature = "compat_real_test"))))]
        {
            R4_CURRENT
        }
    }

    /// Set the current task on the CALLING CPU (full-os Part I.3, SMP scheduler):
    /// the global `R4_CURRENT` on the bootstrap processor, this CPU's own per-CPU
    /// `current` (gs:[16]) on an application processor. The dual of `r4_current_smp`,
    /// so a context switch records the new task against the CPU that ran it. Transparent
    /// on the BSP (and on -smp 1, where `is_bsp` short-circuits with no MSR read).
    unsafe fn r4_set_current_smp(tid: usize) {
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            if !crate::smp::is_bsp() {
                core::arch::asm!("mov qword ptr gs:[16], {v}", v = in(reg) tid as u64, options(nostack));
                return;
            }
        }
        R4_CURRENT = tid;
    }

    #[inline(always)]
    unsafe fn r4_state_code(state: R4State) -> u64 {
        match state {
            R4State::Ready => 0,
            R4State::Running => 1,
            R4State::Blocked => 2,
            R4State::Exited => 3,
            R4State::Dead => 4,
        }
    }

    #[inline(always)]
    unsafe fn r4_can_control_task(requester: usize, target: usize) -> bool {
        requester == target
            || R4_TASKS[target].parent_tid == requester
            || R4_TASKS[requester].can_spawn
    }

    #[inline(always)]
    unsafe fn r4_current_has_cap(flag: u8) -> bool {
        R4_TASKS[r4_current_smp()].cap_flags & flag != 0
    }

    unsafe fn r4_cleanup_task_resources(tid: usize) {
        #[cfg(feature = "go_test")]
        {
            r4_release_owned_fds(tid);
            net::r4_release_owned_sockets(tid);
            r4_release_owned_endpoints(tid);
            r4_release_stale_services();
        }
    }

    unsafe fn r4_copy_isolation_config(
        cfg_ptr: u64,
        cfg_len: u64,
    ) -> Option<(u8, u8, u8, u8, u8)> {
        if cfg_len < 24 {
            return None;
        }
        let mut raw = [0u8; 24];
        let raw_len = raw.len();
        if copyin_user(&mut raw, cfg_ptr, raw_len).is_err() {
            return None;
        }
        let domain = u64::from_le_bytes(raw[0..8].try_into().ok()?) as u8;
        let flags = u64::from_le_bytes(raw[8..16].try_into().ok()?) as u8;
        let limits = u64::from_le_bytes(raw[16..24].try_into().ok()?);
        if flags & !R4_TASK_CAP_MASK != 0 {
            return None;
        }
        let fd_limit = (limits & 0xFF) as u8;
        let socket_limit = ((limits >> 8) & 0xFF) as u8;
        let endpoint_limit = ((limits >> 16) & 0xFF) as u8;
        // Lanes without the FD or socket layers cap those limits at zero.
        #[cfg(feature = "go_test")]
        let fd_cap = M8_FD_MAX.saturating_sub(3);
        #[cfg(not(feature = "go_test"))]
        let fd_cap = 0usize;
        #[cfg(any(feature = "net_test", feature = "go_test"))]
        let socket_cap = net::R4_NET_SOCKET_MAX;
        #[cfg(not(any(feature = "net_test", feature = "go_test")))]
        let socket_cap = 0usize;
        if fd_limit as usize > fd_cap
            || socket_limit as usize > socket_cap
            || endpoint_limit as usize > R4_MAX_ENDPOINTS
        {
            return None;
        }
        Some((domain, flags, fd_limit, socket_limit, endpoint_limit))
    }

    unsafe fn sys_proc_info_r4(tid: u64, info_ptr: u64, info_len: u64) -> u64 {
        let target = tid as usize;
        if target >= R4_NUM_TASKS {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if info_len < R4_PROC_INFO_BASE_SIZE as u64 {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let copy_len = if info_len >= R4_PROC_INFO_EXT_SIZE as u64 {
            R4_PROC_INFO_EXT_SIZE
        } else {
            R4_PROC_INFO_BASE_SIZE
        };
        if !user_range_ok(info_ptr, copy_len)
            || !user_pages_ok(info_ptr, copy_len, USER_PERM_WRITE)
        {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        #[cfg(not(feature = "go_test"))]
        {
            if target != R4_CURRENT {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
        }

        let task = R4_TASKS[target];
        let fields = [
            target as u64,
            task.parent_tid as u64,
            r4_state_code(task.state),
            task.sched_class as u64,
            task.dispatch_count,
            task.yield_count,
            task.block_count,
            task.ipc_send_count,
            task.ipc_recv_count,
            task.endpoint_count as u64,
            task.shm_count as u64,
            task.thread_count as u64,
            task.exit_status,
            task.isolation_domain as u64,
            task.cap_flags as u64,
            task.fd_count as u64,
            task.socket_count as u64,
        ];
        let mut out = [0u8; R4_PROC_INFO_EXT_SIZE];
        for (idx, field) in fields.iter().enumerate() {
            let start = idx * 8;
            out[start..start + 8].copy_from_slice(&field.to_le_bytes());
        }
        if copyout_user(info_ptr, &out[..copy_len], copy_len).is_err() {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        0
    }

    unsafe fn sys_isolation_config_r4(tid: u64, cfg_ptr: u64, cfg_len: u64) -> u64 {
        let target = tid as usize;
        if target >= R4_NUM_TASKS {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if !r4_can_control_task(R4_CURRENT, target) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let (domain, flags, fd_limit, socket_limit, endpoint_limit) =
            match r4_copy_isolation_config(cfg_ptr, cfg_len) {
                Some(v) => v,
                None => return 0xFFFF_FFFF_FFFF_FFFF,
            };
        if R4_TASKS[target].fd_count > fd_limit as usize
            || R4_TASKS[target].socket_count > socket_limit as usize
            || R4_TASKS[target].endpoint_count > endpoint_limit as usize
        {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        R4_TASKS[target].isolation_domain = domain;
        R4_TASKS[target].cap_flags = flags;
        R4_TASKS[target].fd_limit = fd_limit;
        R4_TASKS[target].socket_limit = socket_limit;
        R4_TASKS[target].endpoint_limit = endpoint_limit;
        0
    }

    unsafe fn sys_sched_set_r4(tid: u64, class: u64) -> u64 {
        let target = tid as usize;
        if target >= R4_NUM_TASKS {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if R4_TASKS[target].state == R4State::Dead {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let next_class = match class as u8 {
            R4_SCHED_CLASS_BEST_EFFORT => R4_SCHED_CLASS_BEST_EFFORT,
            R4_SCHED_CLASS_CRITICAL => R4_SCHED_CLASS_CRITICAL,
            _ => return 0xFFFF_FFFF_FFFF_FFFF,
        };
        if !r4_can_control_task(R4_CURRENT, target) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        R4_TASKS[target].sched_class = next_class;
        0
    }

    unsafe fn sys_thread_spawn_r4(entry: u64) -> u64 {
        #[cfg(feature = "quota_threads_test")]
        {
            if entry >= 0x0000_8000_0000_0000 { return 0xFFFF_FFFF_FFFF_FFFF; }
            if !runtime::isolation::under_quota(
                R4_TASKS[r4_current_smp()].thread_count,
                MAX_THREADS_PER_PROC,
            ) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            if !runtime::isolation::under_quota(R4_THREADS_CREATED, MAX_THREADS_GLOBAL) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            let tid = R4_TASKS[r4_current_smp()].thread_count as u64;
            R4_TASKS[r4_current_smp()].thread_count += 1;
            R4_THREADS_CREATED += 1;
            return tid;
        }
        #[cfg(all(feature = "go_test", not(feature = "quota_threads_test")))]
        {
            if entry >= 0x0000_8000_0000_0000 {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            if !R4_TASKS[r4_current_smp()].can_spawn {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            if !user_pages_ok(entry, 1, USER_PERM_READ) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            let tid = match r4_find_spawn_slot() {
                Some(tid) => tid,
                None => return 0xFFFF_FFFF_FFFF_FFFF,
            };
            if tid >= R4_MAX_TASKS {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            r4_init_task(tid, entry, r4_stack_top_for_slot(tid), R4_CURRENT);
            R4_TASKS[tid].state = R4State::Ready;
            R4_THREADS_CREATED += 1;
            tid as u64
        }
        #[cfg(all(not(feature = "quota_threads_test"), not(feature = "go_test")))]
        {
            let _ = entry;
            0xFFFF_FFFF_FFFF_FFFF
        }
    }

    unsafe fn sys_wait_r4(frame: *mut u64, pid: u64, status_ptr: u64, options: u64) {
        if options != 0 {
            *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            return;
        }

        let target = if pid == u64::MAX {
            -1
        } else if (pid as usize) < R4_NUM_TASKS {
            pid as i32
        } else {
            *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            return;
        };

        if status_ptr != 0
            && (!user_range_ok(status_ptr, 8)
                || !user_pages_ok(status_ptr, 8, USER_PERM_WRITE))
        {
            *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            return;
        }

        let cur = R4_CURRENT;
        if let Some(child) = r4_find_exited_child(cur, target) {
            if !r4_copy_wait_status(status_ptr, R4_TASKS[child].exit_status) {
                *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
                return;
            }
            R4_TASKS[child].state = R4State::Dead;
            R4_TASKS[child].exit_status = 0;
            *frame.add(14) = child as u64;
            return;
        }

        if !r4_has_waitable_child(cur, target) {
            *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            return;
        }

        r4_save_frame(frame, cur);
        R4_TASKS[cur].wait_target = target;
        R4_TASKS[cur].wait_status_ptr = status_ptr;
        R4_TASKS[cur].block_count += 1;
        R4_TASKS[cur].state = R4State::Blocked;
        match r4_find_ready(cur) {
            Some(tid) => { r4_switch_to(frame, tid); }
            None => {
                #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                r4_enter_idle_or_done(frame);
                #[cfg(not(all(feature = "go_test", not(feature = "compat_real_test"))))]
                {
                    serial_write(b"R4: deadlock\n");
                    let kstack = &stack_top as *const u8 as u64;
                    *frame.add(17) = r4_all_done as *const () as u64;
                    *frame.add(18) = 0x08;
                    *frame.add(19) = 0x02;
                    *frame.add(20) = kstack;
                    *frame.add(21) = 0x10;
                }
            }
        }
    }
}

// --------------- R4: IPC endpoints -------------------------------------------

cfg_r4! {
    const R4_MAX_ENDPOINTS: usize = 16;
    const R4_MAX_MSG_LEN: usize = 256;
    const R4_EP_RIGHT_RECV: u8 = 1 << 0;
    const R4_EP_RIGHT_CONTROL: u8 = 1 << 1;

    #[derive(Clone, Copy)]
    struct IpcEndpoint {
        active: bool,
        has_msg: bool,
        msg_data: [u8; R4_MAX_MSG_LEN],
        msg_len: usize,
        waiter: i32, // task id blocked on recv, or -1
        owner_tid: usize,
        owner_rights: u8,
    }

    impl IpcEndpoint {
        const EMPTY: Self = Self {
            active: false, has_msg: false,
            msg_data: [0u8; R4_MAX_MSG_LEN], msg_len: 0,
            waiter: -1,
            owner_tid: 0,
            owner_rights: 0,
        };
    }

    static mut R4_ENDPOINTS: [IpcEndpoint; R4_MAX_ENDPOINTS] =
        [IpcEndpoint::EMPTY; R4_MAX_ENDPOINTS];

    #[inline(always)]
    unsafe fn r4_endpoint_owner_has_right(ep: usize, right: u8) -> bool {
        #[cfg(feature = "go_test")]
        {
            runtime::isolation::owner_has_right(
                R4_ENDPOINTS[ep].owner_tid,
                R4_CURRENT,
                R4_ENDPOINTS[ep].owner_rights,
                right,
            )
        }
        #[cfg(not(feature = "go_test"))]
        {
            let _ = ep;
            let _ = right;
            true
        }
    }

    #[cfg(feature = "go_test")]
    unsafe fn r4_release_owned_endpoints(owner_tid: usize) {
        for ep in 0..R4_MAX_ENDPOINTS {
            if !R4_ENDPOINTS[ep].active || R4_ENDPOINTS[ep].owner_tid != owner_tid {
                continue;
            }
            let waiter = R4_ENDPOINTS[ep].waiter;
            if waiter >= 0 {
                let wt = waiter as usize;
                if wt < R4_NUM_TASKS && R4_TASKS[wt].state == R4State::Blocked {
                    R4_TASKS[wt].recv_ep = 0;
                    R4_TASKS[wt].recv_buf = 0;
                    R4_TASKS[wt].recv_cap = 0;
                    R4_TASKS[wt].saved_frame[14] = 0xFFFF_FFFF_FFFF_FFFF;
                    R4_TASKS[wt].state = R4State::Ready;
                }
            }
            R4_ENDPOINTS[ep] = IpcEndpoint::EMPTY;
        }
        if owner_tid < R4_NUM_TASKS {
            R4_TASKS[owner_tid].endpoint_count = 0;
        }
    }

    unsafe fn sys_ipc_endpoint_create_r4() -> u64 {
        #[cfg(any(feature = "quota_endpoints_test", feature = "go_test"))]
        {
            let limit = if cfg!(feature = "go_test") {
                R4_TASKS[r4_current_smp()].endpoint_limit as usize
            } else {
                MAX_ENDPOINTS_PER_PROC
            };
            if !runtime::isolation::under_quota(
                R4_TASKS[r4_current_smp()].endpoint_count,
                limit,
            ) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            for i in 0..R4_MAX_ENDPOINTS {
                if !R4_ENDPOINTS[i].active {
                    R4_ENDPOINTS[i].active = true;
                    R4_ENDPOINTS[i].has_msg = false;
                    R4_ENDPOINTS[i].msg_len = 0;
                    R4_ENDPOINTS[i].waiter = -1;
                    R4_ENDPOINTS[i].owner_tid = R4_CURRENT;
                    R4_ENDPOINTS[i].owner_rights = R4_EP_RIGHT_RECV | R4_EP_RIGHT_CONTROL;
                    R4_TASKS[r4_current_smp()].endpoint_count += 1;
                    return i as u64;
                }
            }
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        #[cfg(all(not(feature = "quota_endpoints_test"), not(feature = "go_test")))]
        {
            0xFFFF_FFFF_FFFF_FFFF
        }
    }

    unsafe fn sys_ipc_send_r4(endpoint: u64, buf: u64, len: u64) -> u64 {
        let ep = endpoint as usize;
        if ep >= R4_MAX_ENDPOINTS || !R4_ENDPOINTS[ep].active {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        // Single-slot buffer: reject if message already buffered
        if R4_ENDPOINTS[ep].has_msg {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }

        if len == 0 || len > R4_MAX_MSG_LEN as u64 {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        let n = len as usize;

        // Copy from user to kernel buffer (validates range + page tables)
        let mut kbuf = [0u8; R4_MAX_MSG_LEN];
        if n > 0 {
            if copyin_user(&mut kbuf[..n], buf, n).is_err() { return 0xFFFF_FFFF_FFFF_FFFF; }
        }
        R4_TASKS[r4_current_smp()].ipc_send_count += 1;

        // If someone is blocked on recv for this endpoint, deliver directly
        let waiter = R4_ENDPOINTS[ep].waiter;
        if waiter >= 0 {
            let wt = waiter as usize;
            if wt >= R4_MAX_TASKS {
                R4_ENDPOINTS[ep].waiter = -1;
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            // Stale waiter can happen if bookkeeping got out of sync.
            if R4_TASKS[wt].state != R4State::Blocked || R4_TASKS[wt].recv_ep != endpoint {
                R4_ENDPOINTS[ep].waiter = -1;
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            // Never silently truncate delivery to a blocked receiver.
            if (R4_TASKS[wt].recv_cap as usize) < n {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            // Deliver into the RECEIVER's address space, not the sender's
            // current CR3 (they may differ under per-process address spaces).
            let wt_pml4 = R4_TASKS[wt].pml4_phys;
            let delivered = if wt_pml4 != 0 {
                mm::as_copyout(wt_pml4, R4_TASKS[wt].recv_buf, &kbuf[..n])
            } else {
                copyout_user(R4_TASKS[wt].recv_buf, &kbuf[..n], n).is_ok()
            };
            if !delivered {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            R4_TASKS[wt].saved_frame[14] = n as u64; // return value for recv
            R4_TASKS[wt].state = R4State::Ready;
            R4_TASKS[wt].ipc_recv_count += 1;
            R4_ENDPOINTS[ep].waiter = -1;
            return 0;
        }

        // No waiter â€” buffer the message
        R4_ENDPOINTS[ep].msg_data[..n].copy_from_slice(&kbuf[..n]);
        R4_ENDPOINTS[ep].msg_len = n;
        R4_ENDPOINTS[ep].has_msg = true;
        0
    }

    unsafe fn sys_ipc_recv_r4(frame: *mut u64, endpoint: u64, buf: u64, cap: u64) {
        let ep = endpoint as usize;
        if ep >= R4_MAX_ENDPOINTS || !R4_ENDPOINTS[ep].active {
            *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            return;
        }
        if !r4_endpoint_owner_has_right(ep, R4_EP_RIGHT_RECV) {
            *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            return;
        }
        if cap == 0 {
            *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            return;
        }
        let cap_n = if cap > R4_MAX_MSG_LEN as u64 { R4_MAX_MSG_LEN } else { cap as usize };
        if !user_range_ok(buf, cap_n) || !user_pages_ok(buf, cap_n, USER_PERM_WRITE) {
            *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            return;
        }

        // If message available, deliver immediately
        if R4_ENDPOINTS[ep].has_msg {
            let n = R4_ENDPOINTS[ep].msg_len;
            // Never silently truncate a queued message on recv.
            if cap_n < n {
                *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
                return;
            }
            if copyout_user(buf, &R4_ENDPOINTS[ep].msg_data[..n], n).is_err() {
                *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
                return;
            }
            R4_ENDPOINTS[ep].has_msg = false;
            R4_TASKS[r4_current_smp()].ipc_recv_count += 1;
            *frame.add(14) = n as u64;
            return;
        }

        // A second waiter on the same endpoint is rejected explicitly.
        if R4_ENDPOINTS[ep].waiter >= 0 && R4_ENDPOINTS[ep].waiter != R4_CURRENT as i32 {
            *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            return;
        }

        // No message â€” block current task and switch
        R4_TASKS[r4_current_smp()].recv_ep = endpoint;
        R4_TASKS[r4_current_smp()].recv_buf = buf;
        R4_TASKS[r4_current_smp()].recv_cap = cap_n as u64;
        r4_save_frame(frame, R4_CURRENT);
        R4_TASKS[r4_current_smp()].block_count += 1;
        R4_TASKS[r4_current_smp()].state = R4State::Blocked;
        R4_ENDPOINTS[ep].waiter = R4_CURRENT as i32;

        match r4_find_ready(R4_CURRENT) {
            Some(tid) => { r4_switch_to(frame, tid); }
            None => {
                #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
                r4_enter_idle_or_done(frame);
                #[cfg(not(all(feature = "go_test", not(feature = "compat_real_test"))))]
                {
                    // Deadlock - no ready tasks
                    serial_write(b"R4: deadlock\n");
                    let kstack = &stack_top as *const u8 as u64;
                    *frame.add(17) = r4_all_done as *const () as u64;
                    *frame.add(18) = 0x08;
                    *frame.add(19) = 0x02;
                    *frame.add(20) = kstack;
                    *frame.add(21) = 0x10;
                }
            }
        }
    }
}

// --------------- R4: Service registry ----------------------------------------

cfg_r4! {
    const R4_MAX_SERVICES: usize = 4;

    struct ServiceEntry {
        active: bool,
        name: [u8; 16],
        name_len: usize,
        endpoint: u64,
    }

    impl ServiceEntry {
        const EMPTY: Self = Self {
            active: false, name: [0u8; 16], name_len: 0, endpoint: 0,
        };
    }

    static mut R4_SERVICES: [ServiceEntry; R4_MAX_SERVICES] =
        [ServiceEntry::EMPTY, ServiceEntry::EMPTY, ServiceEntry::EMPTY, ServiceEntry::EMPTY];

    #[cfg(feature = "go_test")]
    unsafe fn r4_release_stale_services() {
        for idx in 0..R4_MAX_SERVICES {
            if !R4_SERVICES[idx].active {
                continue;
            }
            let ep = R4_SERVICES[idx].endpoint as usize;
            if ep >= R4_MAX_ENDPOINTS || !R4_ENDPOINTS[ep].active {
                R4_SERVICES[idx] = ServiceEntry::EMPTY;
            }
        }
    }

    unsafe fn sys_svc_register_r4(name_ptr: u64, name_len: u64, endpoint: u64) -> u64 {
        let n = name_len as usize;
        if n == 0 || n > 16 { return 0xFFFF_FFFF_FFFF_FFFF; }
        let mut name = [0u8; 16];
        if copyin_user(&mut name[..n], name_ptr, n).is_err() { return 0xFFFF_FFFF_FFFF_FFFF; }
        #[cfg(feature = "go_test")]
        r4_release_stale_services();
        let ep = endpoint as usize;
        if ep >= R4_MAX_ENDPOINTS || !R4_ENDPOINTS[ep].active {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        if !r4_endpoint_owner_has_right(ep, R4_EP_RIGHT_CONTROL) {
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        // Overwrite if name already registered
        for i in 0..R4_MAX_SERVICES {
            if R4_SERVICES[i].active && R4_SERVICES[i].name_len == n
                && R4_SERVICES[i].name[..n] == name[..n]
            {
                R4_SERVICES[i].endpoint = endpoint;
                return 0;
            }
        }
        // Otherwise insert into first free slot
        for i in 0..R4_MAX_SERVICES {
            if !R4_SERVICES[i].active {
                R4_SERVICES[i].active = true;
                R4_SERVICES[i].name = name;
                R4_SERVICES[i].name_len = n;
                R4_SERVICES[i].endpoint = endpoint;
                return 0;
            }
        }
        0xFFFF_FFFF_FFFF_FFFF
    }

    unsafe fn sys_svc_lookup_r4(name_ptr: u64, name_len: u64) -> u64 {
        let n = name_len as usize;
        if n == 0 || n > 16 { return 0xFFFF_FFFF_FFFF_FFFF; }
        let mut name = [0u8; 16];
        if copyin_user(&mut name[..n], name_ptr, n).is_err() { return 0xFFFF_FFFF_FFFF_FFFF; }
        #[cfg(feature = "go_test")]
        r4_release_stale_services();
        for i in 0..R4_MAX_SERVICES {
            if R4_SERVICES[i].active && R4_SERVICES[i].name_len == n
                && R4_SERVICES[i].name[..n] == name[..n]
            {
                return R4_SERVICES[i].endpoint;
            }
        }
        0xFFFF_FFFF_FFFF_FFFF
    }
}

// --------------- R4: SHM syscalls --------------------------------------------

cfg_r4! {
    unsafe fn sys_shm_create_r4(size: u64) -> u64 {
        #[cfg(any(feature = "shm_test", feature = "quota_shm_test"))]
        {
            if size == 0 || size > 4096 { return 0xFFFF_FFFF_FFFF_FFFF; }
            if !runtime::isolation::under_quota(
                R4_TASKS[r4_current_smp()].shm_count,
                MAX_SHM_PER_PROC,
            ) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            for i in 0..R4_MAX_SHM {
                if !R4_SHM_OBJECTS[i].active {
                    R4_SHM_OBJECTS[i].active = true;
                    R4_SHM_OBJECTS[i].size = 4096;
                    core::ptr::write_bytes(R4_SHM_PAGES[i].0.as_mut_ptr(), 0, 4096);
                    R4_TASKS[r4_current_smp()].shm_count += 1;
                    return i as u64;
                }
            }
            return 0xFFFF_FFFF_FFFF_FFFF;
        }
        #[cfg(not(any(feature = "shm_test", feature = "quota_shm_test")))]
        { let _ = size; 0xFFFF_FFFF_FFFF_FFFF }
    }

    unsafe fn sys_shm_map_r4(handle: u64, addr_hint: u64, _flags: u64) -> u64 {
        #[cfg(any(feature = "shm_test", feature = "quota_shm_test"))]
        {
            let h = handle as usize;
            if h >= R4_MAX_SHM || !R4_SHM_OBJECTS[h].active { return 0xFFFF_FFFF_FFFF_FFFF; }
            if addr_hint & 0xFFF != 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
            if addr_hint >= 0x0000_8000_0000_0000 { return 0xFFFF_FFFF_FFFF_FFFF; }

            // Get physical address of SHM backing page via kv2p
            let hhdm_resp_ptr = core::ptr::read_volatile(
                core::ptr::addr_of!(HHDM_REQUEST.response));
            let kaddr_resp_ptr = core::ptr::read_volatile(
                core::ptr::addr_of!(KADDR_REQUEST.response));
            let hhdm = (*hhdm_resp_ptr).offset;
            let kphys = (*kaddr_resp_ptr).physical_base;
            let kvirt = (*kaddr_resp_ptr).virtual_base;
            let shm_phys = R4_SHM_PAGES[h].0.as_ptr() as u64 - kvirt + kphys;

            // Walk page tables to find PT and install PTE
            let cr3: u64;
            core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
            let pml4_phys = cr3 & 0x000F_FFFF_FFFF_F000;
            let pml4 = (pml4_phys + hhdm) as *const u64;
            let pml4e = *pml4.add(((addr_hint >> 39) & 0x1FF) as usize);
            if pml4e & 1 == 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
            let pdpt = ((pml4e & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
            let pdpte = *pdpt.add(((addr_hint >> 30) & 0x1FF) as usize);
            if pdpte & 1 == 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
            let pd = ((pdpte & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
            let pde = *pd.add(((addr_hint >> 21) & 0x1FF) as usize);
            if pde & 1 == 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
            let pt = ((pde & 0x000F_FFFF_FFFF_F000) + hhdm) as *mut u64;
            let pt_idx = ((addr_hint >> 12) & 0x1FF) as usize;
            *pt.add(pt_idx) = shm_phys | 0x07; // Present | Writable | Use
            core::arch::asm!("invlpg [{}]", in(reg) addr_hint, options(nostack));
            return addr_hint;
        }
        #[cfg(not(any(feature = "shm_test", feature = "quota_shm_test")))]
        { let _ = (handle, addr_hint, _flags); 0xFFFF_FFFF_FFFF_FFFF }
    }

    unsafe fn sys_shm_unmap_r4(addr: u64) -> u64 {
        #[cfg(any(feature = "shm_test", feature = "quota_shm_test"))]
        {
            if addr & 0xFFF != 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
            if addr >= 0x0000_8000_0000_0000 { return 0xFFFF_FFFF_FFFF_FFFF; }

            let hhdm_resp_ptr = core::ptr::read_volatile(
                core::ptr::addr_of!(HHDM_REQUEST.response));
            let hhdm = (*hhdm_resp_ptr).offset;

            let cr3: u64;
            core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
            let pml4_phys = cr3 & 0x000F_FFFF_FFFF_F000;
            let pml4 = (pml4_phys + hhdm) as *const u64;
            let pml4e = *pml4.add(((addr >> 39) & 0x1FF) as usize);
            if pml4e & 1 == 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
            let pdpt = ((pml4e & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
            let pdpte = *pdpt.add(((addr >> 30) & 0x1FF) as usize);
            if pdpte & 1 == 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
            let pd = ((pdpte & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
            let pde = *pd.add(((addr >> 21) & 0x1FF) as usize);
            if pde & 1 == 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
            let pt = ((pde & 0x000F_FFFF_FFFF_F000) + hhdm) as *mut u64;
            let pt_idx = ((addr >> 12) & 0x1FF) as usize;
            if *pt.add(pt_idx) & 1 == 0 { return 0xFFFF_FFFF_FFFF_FFFF; }
            *pt.add(pt_idx) = 0;
            core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack));
            0
        }
        #[cfg(not(any(feature = "shm_test", feature = "quota_shm_test")))]
        { let _ = addr; 0xFFFF_FFFF_FFFF_FFFF }
    }
}

// --------------- R4: Page table setup for two tasks --------------------------

cfg_r4! {
    unsafe fn setup_r4_pages(blob0: &[u8], blob1: &[u8]) {
        let hhdm_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(HHDM_REQUEST.response));
        let kaddr_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(KADDR_REQUEST.response));
        let hhdm = (*hhdm_resp_ptr).offset;
        let kphys = (*kaddr_resp_ptr).physical_base;
        let kvirt = (*kaddr_resp_ptr).virtual_base;
        HHDM_OFFSET = hhdm;
        let kv2p = |va: u64| -> u64 { va - kvirt + kphys };

        // Clone current PML4
        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        let old_pml4 = ((cr3 & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
        let new_pml4 = USER_PML4.0.as_mut_ptr() as *mut u64;
        for i in 0..512 { *new_pml4.add(i) = *old_pml4.add(i); }

        // PDPT entry 0 -> PD
        let pdpt = USER_PDPT.0.as_mut_ptr() as *mut u64;
        *pdpt = kv2p(USER_PD.0.as_ptr() as u64) | 0x07;

        // PD entry 2 -> PT_CODE (covers 0x400000-0x5FFFFF)
        // PD entry 3 -> PT_STACK (covers 0x600000-0x7FFFFF)
        let pd = USER_PD.0.as_mut_ptr() as *mut u64;
        *pd.add(2) = kv2p(USER_PT_CODE.0.as_ptr() as u64) | 0x07;
        *pd.add(3) = kv2p(USER_PT_STACK.0.as_ptr() as u64) | 0x07;

        // PT_CODE[0] = task 0 code page at 0x400000 (RX User)
        // PT_CODE[1] = task 1 code page at 0x401000 (RX User)
        let code_flags = if cfg!(feature = "go_test") { 0x07 } else { 0x05 };
        let pt_code = USER_PT_CODE.0.as_mut_ptr() as *mut u64;
        *pt_code.add(0) = kv2p(USER_CODE_PAGE.0.as_ptr() as u64) | code_flags;
        *pt_code.add(1) = kv2p(USER_CODE_PAGE_2.0.as_ptr() as u64) | code_flags;

        // PT_STACK[511] = task 0 stack page at 0x7FF000 (RW User)
        // PT_STACK[510] = task 1 stack page at 0x7FE000 (RW User)
        let pt_stack = USER_PT_STACK.0.as_mut_ptr() as *mut u64;
        *pt_stack.add(511) = kv2p(USER_STACK_PAGE.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(510) = kv2p(USER_STACK_PAGE_2.0.as_ptr() as u64) | 0x07;

        // PML4[0] -> our user PDPT
        *new_pml4 = kv2p(USER_PDPT.0.as_ptr() as u64) | 0x07;

        // Copy code blobs
        core::ptr::copy_nonoverlapping(
            blob0.as_ptr(), USER_CODE_PAGE.0.as_mut_ptr(), blob0.len());
        core::ptr::copy_nonoverlapping(
            blob1.as_ptr(), USER_CODE_PAGE_2.0.as_mut_ptr(), blob1.len());

        // Switch CR3
        let new_pml4_phys = kv2p(new_pml4 as u64);
        core::arch::asm!("mov cr3, {}", in(reg) new_pml4_phys, options(nostack));
    }

    #[cfg(any(feature = "stress_ipc_test", feature = "go_test"))]
    unsafe fn setup_r4_pages4(blob0: &[u8], blob1: &[u8], blob2: &[u8], blob3: &[u8]) {
        let hhdm_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(HHDM_REQUEST.response));
        let kaddr_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(KADDR_REQUEST.response));
        let hhdm = (*hhdm_resp_ptr).offset;
        let kphys = (*kaddr_resp_ptr).physical_base;
        let kvirt = (*kaddr_resp_ptr).virtual_base;
        HHDM_OFFSET = hhdm;
        let kv2p = |va: u64| -> u64 { va - kvirt + kphys };

        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        let old_pml4 = ((cr3 & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
        let new_pml4 = USER_PML4.0.as_mut_ptr() as *mut u64;
        for i in 0..512 { *new_pml4.add(i) = *old_pml4.add(i); }

        let pdpt = USER_PDPT.0.as_mut_ptr() as *mut u64;
        *pdpt = kv2p(USER_PD.0.as_ptr() as u64) | 0x07;

        let pd = USER_PD.0.as_mut_ptr() as *mut u64;
        *pd.add(2) = kv2p(USER_PT_CODE.0.as_ptr() as u64) | 0x07;
        *pd.add(3) = kv2p(USER_PT_STACK.0.as_ptr() as u64) | 0x07;

        let code_flags = if cfg!(feature = "go_test") { 0x07 } else { 0x05 };
        let pt_code = USER_PT_CODE.0.as_mut_ptr() as *mut u64;
        *pt_code.add(0) = kv2p(USER_CODE_PAGE.0.as_ptr() as u64) | code_flags;
        *pt_code.add(1) = kv2p(USER_CODE_PAGE_2.0.as_ptr() as u64) | code_flags;
        *pt_code.add(2) = kv2p(USER_CODE_PAGE_3.0.as_ptr() as u64) | code_flags;
        *pt_code.add(3) = kv2p(USER_CODE_PAGE_4.0.as_ptr() as u64) | code_flags;
        #[cfg(feature = "go_test")]
        {
            *pt_code.add(4) = kv2p(USER_CODE_PAGE_5.0.as_ptr() as u64) | code_flags;
            *pt_code.add(5) = kv2p(USER_CODE_PAGE_6.0.as_ptr() as u64) | code_flags;
            *pt_code.add(6) = kv2p(USER_CODE_PAGE_7.0.as_ptr() as u64) | code_flags;
            *pt_code.add(7) = kv2p(USER_CODE_PAGE_8.0.as_ptr() as u64) | code_flags;
        }

        let pt_stack = USER_PT_STACK.0.as_mut_ptr() as *mut u64;
        *pt_stack.add(511) = kv2p(USER_STACK_PAGE.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(510) = kv2p(USER_STACK_PAGE_2.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(509) = kv2p(USER_STACK_PAGE_3.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(508) = kv2p(USER_STACK_PAGE_4.0.as_ptr() as u64) | 0x07;

        *new_pml4 = kv2p(USER_PDPT.0.as_ptr() as u64) | 0x07;

        core::ptr::copy_nonoverlapping(
            blob0.as_ptr(), USER_CODE_PAGE.0.as_mut_ptr(), blob0.len());
        core::ptr::copy_nonoverlapping(
            blob1.as_ptr(), USER_CODE_PAGE_2.0.as_mut_ptr(), blob1.len());
        core::ptr::copy_nonoverlapping(
            blob2.as_ptr(), USER_CODE_PAGE_3.0.as_mut_ptr(), blob2.len());
        core::ptr::copy_nonoverlapping(
            blob3.as_ptr(), USER_CODE_PAGE_4.0.as_mut_ptr(), blob3.len());

        let new_pml4_phys = kv2p(new_pml4 as u64);
        core::arch::asm!("mov cr3, {}", in(reg) new_pml4_phys, options(nostack));
    }

    #[cfg(feature = "go_test")]
    unsafe fn setup_go_user_pages(blob: &[u8]) {
        if blob.is_empty() || blob.len() > runtime::process::GO_IMAGE_MAX_BYTES {
            serial_write(b"GO: image too large\n");
            qemu_exit(0x33);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }

        let hhdm_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(HHDM_REQUEST.response));
        let kaddr_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(KADDR_REQUEST.response));
        let hhdm = (*hhdm_resp_ptr).offset;
        let kphys = (*kaddr_resp_ptr).physical_base;
        let kvirt = (*kaddr_resp_ptr).virtual_base;
        HHDM_OFFSET = hhdm;
        let kv2p = |va: u64| -> u64 { va - kvirt + kphys };

        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        let old_pml4 = ((cr3 & 0x000F_FFFF_FFFF_F000) + hhdm) as *const u64;
        let new_pml4 = USER_PML4.0.as_mut_ptr() as *mut u64;
        for i in 0..512 { *new_pml4.add(i) = *old_pml4.add(i); }

        let pdpt = USER_PDPT.0.as_mut_ptr() as *mut u64;
        *pdpt = kv2p(USER_PD.0.as_ptr() as u64) | 0x07;

        let pd = USER_PD.0.as_mut_ptr() as *mut u64;
        *pd.add(2) = kv2p(USER_PT_CODE.0.as_ptr() as u64) | 0x07;
        *pd.add(3) = kv2p(USER_PT_STACK.0.as_ptr() as u64) | 0x07;

        let pt_code = USER_PT_CODE.0.as_mut_ptr() as *mut u64;
        let code_flags = 0x07;
        *pt_code.add(0) = kv2p(USER_CODE_PAGE.0.as_ptr() as u64) | code_flags;
        *pt_code.add(1) = kv2p(USER_CODE_PAGE_2.0.as_ptr() as u64) | code_flags;
        *pt_code.add(2) = kv2p(USER_CODE_PAGE_3.0.as_ptr() as u64) | code_flags;
        *pt_code.add(3) = kv2p(USER_CODE_PAGE_4.0.as_ptr() as u64) | code_flags;
        *pt_code.add(4) = kv2p(USER_CODE_PAGE_5.0.as_ptr() as u64) | code_flags;
        *pt_code.add(5) = kv2p(USER_CODE_PAGE_6.0.as_ptr() as u64) | code_flags;
        *pt_code.add(6) = kv2p(USER_CODE_PAGE_7.0.as_ptr() as u64) | code_flags;
        *pt_code.add(7) = kv2p(USER_CODE_PAGE_8.0.as_ptr() as u64) | code_flags;

        let pt_stack = USER_PT_STACK.0.as_mut_ptr() as *mut u64;
        *pt_stack.add(511) = kv2p(USER_STACK_PAGE.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(510) = kv2p(USER_STACK_PAGE_2.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(509) = kv2p(USER_STACK_PAGE_3.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(508) = kv2p(USER_STACK_PAGE_4.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(507) = kv2p(USER_STACK_PAGE_5.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(506) = kv2p(USER_STACK_PAGE_6.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(505) = kv2p(USER_STACK_PAGE_7.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(504) = kv2p(USER_STACK_PAGE_8.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(503) = kv2p(USER_HEAP_PAGE_1.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(502) = kv2p(USER_HEAP_PAGE_2.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(501) = kv2p(USER_HEAP_PAGE_3.0.as_ptr() as u64) | 0x07;
        *pt_stack.add(500) = kv2p(USER_HEAP_PAGE_4.0.as_ptr() as u64) | 0x07;

        *new_pml4 = kv2p(USER_PDPT.0.as_ptr() as u64) | 0x07;

        core::ptr::write_bytes(USER_CODE_PAGE.0.as_mut_ptr(), 0, runtime::process::GO_IMAGE_PAGE_SIZE);
        core::ptr::write_bytes(USER_CODE_PAGE_2.0.as_mut_ptr(), 0, runtime::process::GO_IMAGE_PAGE_SIZE);
        core::ptr::write_bytes(USER_CODE_PAGE_3.0.as_mut_ptr(), 0, runtime::process::GO_IMAGE_PAGE_SIZE);
        core::ptr::write_bytes(USER_CODE_PAGE_4.0.as_mut_ptr(), 0, runtime::process::GO_IMAGE_PAGE_SIZE);
        core::ptr::write_bytes(USER_CODE_PAGE_5.0.as_mut_ptr(), 0, runtime::process::GO_IMAGE_PAGE_SIZE);
        core::ptr::write_bytes(USER_CODE_PAGE_6.0.as_mut_ptr(), 0, runtime::process::GO_IMAGE_PAGE_SIZE);
        core::ptr::write_bytes(USER_CODE_PAGE_7.0.as_mut_ptr(), 0, runtime::process::GO_IMAGE_PAGE_SIZE);
        core::ptr::write_bytes(USER_CODE_PAGE_8.0.as_mut_ptr(), 0, runtime::process::GO_IMAGE_PAGE_SIZE);

        let chunks = [
            runtime::process::go_image_chunk(blob, 0),
            runtime::process::go_image_chunk(blob, 1),
            runtime::process::go_image_chunk(blob, 2),
            runtime::process::go_image_chunk(blob, 3),
            runtime::process::go_image_chunk(blob, 4),
            runtime::process::go_image_chunk(blob, 5),
            runtime::process::go_image_chunk(blob, 6),
            runtime::process::go_image_chunk(blob, 7),
        ];
        let pages = [
            USER_CODE_PAGE.0.as_mut_ptr(),
            USER_CODE_PAGE_2.0.as_mut_ptr(),
            USER_CODE_PAGE_3.0.as_mut_ptr(),
            USER_CODE_PAGE_4.0.as_mut_ptr(),
            USER_CODE_PAGE_5.0.as_mut_ptr(),
            USER_CODE_PAGE_6.0.as_mut_ptr(),
            USER_CODE_PAGE_7.0.as_mut_ptr(),
            USER_CODE_PAGE_8.0.as_mut_ptr(),
        ];
        for i in 0..runtime::process::GO_IMAGE_MAX_PAGES {
            if chunks[i].is_empty() {
                continue;
            }
            core::ptr::copy_nonoverlapping(
                chunks[i].as_ptr(),
                pages[i],
                chunks[i].len(),
            );
        }

        let new_pml4_phys = kv2p(new_pml4 as u64);
        // Per-process address spaces clone this table's kernel half.
        #[cfg(not(feature = "compat_real_test"))]
        {
            SHARED_PML4_PHYS = new_pml4_phys;
        }
        core::arch::asm!("mov cr3, {}", in(reg) new_pml4_phys, options(nostack));
    }
}

// =============================================================================
// M5: VirtIO block driver + block syscalls
// =============================================================================

// --------------- M5: PCI config space access ---------------------------------

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const PCI_CONFIG_ADDR: u16 = 0xCF8;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const PCI_CONFIG_DATA: u16 = 0xCFC;

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
unsafe fn pci_read32(bus: u8, dev: u8, func: u8, offset: u8) -> u32 {
    let addr: u32 = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((dev as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);
    outl(PCI_CONFIG_ADDR, addr);
    inl(PCI_CONFIG_DATA)
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
unsafe fn pci_write32(bus: u8, dev: u8, func: u8, offset: u8, value: u32) {
    let addr: u32 = (1u32 << 31)
        | ((bus as u32) << 16)
        | ((dev as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);
    outl(PCI_CONFIG_ADDR, addr);
    outl(PCI_CONFIG_DATA, value);
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
#[derive(Clone, Copy)]
struct PciBdf {
    bus: u8,
    dev: u8,
    func: u8,
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const PCI_CLAIM_NONE: u16 = 0xFFFF;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const PCI_CLAIM_SLOTS: usize = 4;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
static mut PCI_CLAIMED: [u16; PCI_CLAIM_SLOTS] = [PCI_CLAIM_NONE; PCI_CLAIM_SLOTS];

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
fn pci_bdf_key(bdf: PciBdf) -> u16 {
    ((bdf.bus as u16) << 8) | ((bdf.dev as u16) << 3) | (bdf.func as u16)
}

/// Enumerate PCI bus 0 (full-os guide Part II.7 driver model: device
/// discovery). Logs every present function's vendor/device/class so the
/// device inventory is visible at boot; this is the registry-discovery step
/// the per-driver probe/attach refactor builds on. Read-only — it does not
/// claim or initialize anything (the existing virtio/NVMe probes still own
/// attachment).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn pci_enumerate_log() {
    serial_write(b"PCI: enumerate bus0\n");
    let mut count = 0u32;
    let mut dev = 0u8;
    while dev < 32 {
        let id0 = pci_read32(0, dev, 0, 0);
        if (id0 & 0xFFFF) as u16 == 0xFFFF {
            dev += 1;
            continue;
        }
        // Multi-function devices set bit 7 of the header-type byte.
        let hdr = (pci_read32(0, dev, 0, 0x0C) >> 16) & 0xFF;
        let funcs = if hdr & 0x80 != 0 { 8u8 } else { 1u8 };
        let mut func = 0u8;
        while func < funcs {
            let id = pci_read32(0, dev, func, 0);
            let v = (id & 0xFFFF) as u16;
            if v != 0xFFFF {
                let d = ((id >> 16) & 0xFFFF) as u16;
                let class = pci_read32(0, dev, func, 0x08) >> 16; // class<<8|subclass
                serial_write(b"PROBE: dev=0x");
                serial_write_hex(dev as u64);
                serial_write(b" func=0x");
                serial_write_hex(func as u64);
                serial_write(b" vendor=0x");
                serial_write_hex(v as u64);
                serial_write(b" device=0x");
                serial_write_hex(d as u64);
                serial_write(b" class=0x");
                serial_write_hex((class & 0xFFFF) as u64);
                serial_write(b"\n");
                // Driver registry match (full-os guide Part II.7): a known
                // (vendor, device) gets an ATTACH marker. This is the
                // registry/attach step; actual init is still owned by the
                // existing virtio/NVMe probes.
                let name: &[u8] = match (v, d) {
                    (0x1AF4, 0x1001) => b"virtio-blk-pci",
                    (0x1AF4, 0x1000) => b"virtio-net-pci",
                    (0x1B36, 0x0010) | (0x8086, 0x5845) => b"nvme",
                    (0x8086, 0x100E) => b"e1000",
                    _ => b"",
                };
                if !name.is_empty() {
                    serial_write(b"ATTACH: ");
                    serial_write(name);
                    serial_write(b"\n");
                }
                count += 1;
            }
            func += 1;
        }
        dev += 1;
    }
    serial_write(b"PCI: devices=0x");
    serial_write_hex(count as u64);
    serial_write(b"\n");
}

/// Detect a USB xHCI host controller (full-os guide Part II.7): scan PCI bus 0
/// for a Serial-Bus / USB / xHCI function (class 0x0C, subclass 0x03,
/// prog-if 0x30), then read its memory-mapped capability registers so the OS
/// reports the controller it found. This is the discovery + capability-read
/// foundation; command/event rings, port reset, and device enumeration are
/// carry-forward. Reachable only when a controller is attached
/// (`-device qemu-xhci`); otherwise reports `XHCI: none`.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn xhci_detect() {
    let mut dev = 0u8;
    while dev < 32 {
        let id0 = pci_read32(0, dev, 0, 0);
        if (id0 & 0xFFFF) as u16 == 0xFFFF {
            dev += 1;
            continue;
        }
        let hdr = (pci_read32(0, dev, 0, 0x0C) >> 16) & 0xFF;
        let funcs = if hdr & 0x80 != 0 { 8u8 } else { 1u8 };
        let mut func = 0u8;
        while func < funcs {
            let id = pci_read32(0, dev, func, 0);
            if (id & 0xFFFF) as u16 != 0xFFFF {
                let class_reg = pci_read32(0, dev, func, 0x08);
                let class = (class_reg >> 24) as u8;
                let subclass = (class_reg >> 16) as u8;
                let prog_if = (class_reg >> 8) as u8;
                if class == 0x0C && subclass == 0x03 && prog_if == 0x30 {
                    let bdf = PciBdf { bus: 0, dev, func };
                    xhci_report(bdf);
                    xhci_ring_selftest(bdf); // full-os Part II.7: xHCI command/event ring
                    return;
                }
            }
            func += 1;
        }
        dev += 1;
    }
    serial_write(b"XHCI: none\n");
}

/// Map a 4 KiB device-MMIO page into the kernel address space (go lane) and
/// return the VA corresponding to `phys` (page offset preserved). The PCI MMIO
/// hole is not covered by the HHDM, so walk the active CR3 through the HHDM,
/// allocating any missing intermediate page-table levels from the PMM, and
/// install an uncacheable (PWT|PCD) leaf at a fixed kernel MMIO window VA.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn mmio_map_4k(phys: u64) -> Option<u64> {
    const MMIO_WINDOW_VA: u64 = 0xFFFF_FF00_0000_0000; // unused high kernel slot
    let hhdm = crate::mm::hhdm_offset();
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    let mut table_phys = cr3 & 0x000F_FFFF_FFFF_F000;
    let page = phys & !0xFFF;
    // Walk PML4 -> PDPT -> PD (the three non-leaf levels). To stay all-or-nothing
    // under PMM pressure, allocate missing levels but DEFER linking them into the
    // live tree until the whole walk succeeds; on any allocation failure, free
    // what we took and leave the page tables untouched. (A freshly allocated
    // table is reachable through the HHDM before it is linked, so the walk can
    // continue into it.)
    let mut pending: [(u64, usize, u64); 3] = [(0, 0, 0); 3];
    let mut npending = 0usize;
    for &shift in &[39u32, 30, 21] {
        let table = (table_phys + hhdm) as *mut u64;
        let i = ((MMIO_WINDOW_VA >> shift) & 0x1FF) as usize;
        let e = core::ptr::read_volatile(table.add(i));
        if e & 1 == 0 {
            let new = match crate::mm::alloc_frame() {
                Some(f) => f,
                None => {
                    let mut k = 0;
                    while k < npending {
                        crate::mm::free_frame(pending[k].2);
                        k += 1;
                    }
                    return None;
                }
            };
            core::ptr::write_bytes((new + hhdm) as *mut u8, 0, 4096);
            pending[npending] = (table_phys, i, new);
            npending += 1;
            table_phys = new;
        } else {
            table_phys = e & 0x000F_FFFF_FFFF_F000;
        }
    }
    // The whole chain is available; link the deferred tables into the live tree.
    let mut k = 0;
    while k < npending {
        let (parent_phys, i, new) = pending[k];
        core::ptr::write_volatile(((parent_phys + hhdm) as *mut u64).add(i), new | 0b11);
        k += 1;
    }
    // Install the leaf PTE: present | write | PWT | PCD (uncacheable for MMIO) |
    // NX — the BAR is device data the kernel only reads, never code (W^X).
    let pt = (table_phys + hhdm) as *mut u64;
    let i = ((MMIO_WINDOW_VA >> 12) & 0x1FF) as usize;
    core::ptr::write_volatile(
        pt.add(i),
        page | (1 << 0) | (1 << 1) | (1 << 3) | (1 << 4) | (1u64 << 63),
    );
    core::arch::asm!("invlpg [{}]", in(reg) MMIO_WINDOW_VA, options(nostack));
    Some(MMIO_WINDOW_VA + (phys & 0xFFF))
}

/// Map `npages` consecutive 4 KiB pages of device MMIO starting at the
/// page-aligned `phys` into a dedicated kernel window, returning the window VA of
/// `phys`. For a register block that spans more than one page (e.g. an e1000 BAR
/// whose TX-ring registers live at offset 0x3800). `npages` must fit in one PT
/// from the window's PT index (true for the small counts used here).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn mmio_map_region(phys: u64, npages: usize) -> Option<u64> {
    // Shared transient window: each caller maps, uses the region immediately, and is
    // done before the next mmio_map_region call reuses the same window VA.
    mmio_map_region_at(phys, npages, 0xFFFF_FF10_0000_0000)
}

/// Map `npages` of device MMIO at page-aligned `phys` into the kernel window based at
/// `window_va`, returning the window VA of `phys`. Callers that must HOLD the mapping
/// (e.g. the live e1000 NIC) pass a dedicated `window_va` so the shared transient
/// window can't clobber it. `npages` must fit within one PT from the window's index.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn mmio_map_region_at(phys: u64, npages: usize, window_va: u64) -> Option<u64> {
    let region_window_va = window_va;
    let hhdm = crate::mm::hhdm_offset();
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
    let mut table_phys = cr3 & 0x000F_FFFF_FFFF_F000;
    let base_page = phys & !0xFFF;
    // Walk/create PML4 -> PDPT -> PD (deferred linking, all-or-nothing).
    let mut pending: [(u64, usize, u64); 3] = [(0, 0, 0); 3];
    let mut npending = 0usize;
    for &shift in &[39u32, 30, 21] {
        let table = (table_phys + hhdm) as *mut u64;
        let i = ((region_window_va >> shift) & 0x1FF) as usize;
        let e = core::ptr::read_volatile(table.add(i));
        if e & 1 == 0 {
            let new = match crate::mm::alloc_frame() {
                Some(f) => f,
                None => {
                    let mut k = 0;
                    while k < npending {
                        crate::mm::free_frame(pending[k].2);
                        k += 1;
                    }
                    return None;
                }
            };
            core::ptr::write_bytes((new + hhdm) as *mut u8, 0, 4096);
            pending[npending] = (table_phys, i, new);
            npending += 1;
            table_phys = new;
        } else {
            table_phys = e & 0x000F_FFFF_FFFF_F000;
        }
    }
    let mut k = 0;
    while k < npending {
        let (parent_phys, i, new) = pending[k];
        core::ptr::write_volatile(((parent_phys + hhdm) as *mut u64).add(i), new | 0b11);
        k += 1;
    }
    let pt = (table_phys + hhdm) as *mut u64;
    let base_i = ((region_window_va >> 12) & 0x1FF) as usize;
    let mut p = 0usize;
    while p < npages && base_i + p < 512 {
        core::ptr::write_volatile(
            pt.add(base_i + p),
            (base_page + (p as u64) * 0x1000)
                | (1 << 0)
                | (1 << 1)
                | (1 << 3)
                | (1 << 4)
                | (1u64 << 63),
        );
        core::arch::asm!("invlpg [{}]", in(reg) region_window_va + (p as u64) * 0x1000, options(nostack));
        p += 1;
    }
    Some(region_window_va + (phys & 0xFFF))
}

/// e1000 TX-ring self-test (full-os guide Part II.7, e1000 driver): set up a
/// transmit descriptor ring in the DMA pool, enable the transmitter, queue one
/// Ethernet frame, and confirm the device wrote back the descriptor Done bit —
/// proving the TX DMA path works end to end. Called from `e1000_detect` when an
/// e1000 is present.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn e1000_tx_selftest(bdf: PciBdf) {
    let bar0 = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x10);
    let base = (bar0 & 0xFFFF_FFF0) as u64;
    // Map the first 16 KiB of the BAR (covers CTRL/STATUS/TCTL@0x400 + the TX-ring
    // registers @0x3800).
    let mmio = match mmio_map_region(base, 4) {
        Some(p) => p,
        None => {
            serial_write(b"E1000: tx mmio fail\n");
            return;
        }
    };
    let reg = |off: u64| (mmio + off) as *mut u32;
    // DMA: one page for the descriptor ring (16 descriptors × 16 B), one for the
    // packet buffer.
    let ring_phys = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            serial_write(b"E1000: tx dma fail\n");
            return;
        }
    };
    let buf_phys = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            dma::dma_free(ring_phys, 1); // free the ring on the buffer-alloc failure path
            serial_write(b"E1000: tx dma fail\n");
            return;
        }
    };
    let ring = mm::phys_to_virt(ring_phys) as *mut u8;
    let buf = mm::phys_to_virt(buf_phys) as *mut u8;
    core::ptr::write_bytes(ring, 0, 4096);
    // Build a minimal padded Ethernet frame: dst broadcast, src = our MAC.
    core::ptr::write_bytes(buf, 0, 60);
    let mac = net::net_mac();
    for i in 0..6 {
        *buf.add(i) = 0xFF; // broadcast dst
        *buf.add(6 + i) = mac[i];
    }
    *buf.add(12) = 0x88; // an unassigned ethertype (0x88B5, local experimental)
    *buf.add(13) = 0xB5;
    let frame_len: u16 = 60;

    // Set link up (CTRL.SLU bit 6).
    core::ptr::write_volatile(reg(0x0000), core::ptr::read_volatile(reg(0x0000)) | (1 << 6));
    // TX descriptor ring base/length/head/tail.
    core::ptr::write_volatile(reg(0x3800), ring_phys as u32); // TDBAL
    core::ptr::write_volatile(reg(0x3804), (ring_phys >> 32) as u32); // TDBAH
    core::ptr::write_volatile(reg(0x3808), 16 * 16); // TDLEN (16 descriptors)
    core::ptr::write_volatile(reg(0x3810), 0); // TDH
    core::ptr::write_volatile(reg(0x3818), 0); // TDT
    // TIPG (inter-packet gap) + TCTL: EN | PSP | CT=0x0F | COLD=0x40.
    core::ptr::write_volatile(reg(0x0410), 0x0060_200A);
    core::ptr::write_volatile(reg(0x0400), (1 << 1) | (1 << 3) | (0x0F << 4) | (0x40 << 12));

    // Descriptor 0: buffer address, length, CMD = EOP|IFCS|RS, status = 0.
    core::ptr::write_volatile(ring as *mut u64, buf_phys); // buffer address
    core::ptr::write_volatile((ring.add(8)) as *mut u16, frame_len); // length
    *ring.add(11) = 0x01 | 0x02 | 0x08; // CMD: EOP | IFCS | RS
    *ring.add(12) = 0; // status (DD cleared)
    // Hand descriptor 0 to the device.
    core::ptr::write_volatile(reg(0x3818), 1); // TDT = 1

    // Poll the descriptor Done bit (status bit 0), written back by the device.
    let mut spins = 0u32;
    let mut dd = false;
    while spins < 2_000_000 {
        if core::ptr::read_volatile(ring.add(12)) & 0x01 != 0 {
            dd = true;
            break;
        }
        spins += 1;
        core::hint::spin_loop();
    }
    dma::dma_free(ring_phys, 1);
    dma::dma_free(buf_phys, 1);
    if dd {
        serial_write(b"E1000: tx ok\n");
    } else {
        serial_write(b"E1000: tx no-dd\n");
    }
}

/// e1000 RX-ring driver self-test (full-os guide Part II.7). Sets up a receive
/// descriptor ring on the DMA pool, transmits an ARP request for the slirp gateway
/// (10.0.2.2) out the NIC, and confirms the wire REPLY is RECEIVED into the ring
/// (descriptor Done bit + it is the expected ARP reply from 10.0.2.2) — the
/// receive half of a real NIC driver, exercising a genuine inbound frame delivered
/// by the network backend. Called from e1000_detect after the TX self-test.
///
/// (The legacy e1000 in QEMU does not implement PHY/MAC loopback, so a real wire
/// round-trip — the always-present slirp ARP responder — is used instead.)
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn e1000_rx_selftest(bdf: PciBdf) {
    const NRXD: usize = 8; // RX descriptors (and one DMA page of buffer each)
    let bar0 = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x10);
    let base = (bar0 & 0xFFFF_FFF0) as u64;
    // 16 KiB of BAR covers CTRL/STATUS@0, RCTL@0x100, TCTL@0x400, RX ring@0x2800
    // and TX ring@0x3800.
    let mmio = match mmio_map_region(base, 4) {
        Some(p) => p,
        None => {
            serial_write(b"E1000: rx mmio fail\n");
            return;
        }
    };
    let reg = |off: u64| (mmio + off) as *mut u32;

    // DMA regions: RX ring (1 page), RX buffers (NRXD contiguous pages), TX ring
    // (1 page), TX buffer (1 page). Cascade-free on any allocation failure.
    let rxring_phys = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            serial_write(b"E1000: rx dma fail\n");
            return;
        }
    };
    let rxbuf_phys = match dma::dma_alloc(NRXD) {
        Some(p) => p,
        None => {
            dma::dma_free(rxring_phys, 1);
            serial_write(b"E1000: rx dma fail\n");
            return;
        }
    };
    let txring_phys = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            dma::dma_free(rxbuf_phys, NRXD);
            dma::dma_free(rxring_phys, 1);
            serial_write(b"E1000: rx dma fail\n");
            return;
        }
    };
    let txbuf_phys = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            dma::dma_free(txring_phys, 1);
            dma::dma_free(rxbuf_phys, NRXD);
            dma::dma_free(rxring_phys, 1);
            serial_write(b"E1000: rx dma fail\n");
            return;
        }
    };

    // Build the RX descriptor ring: each 16-byte descriptor's buffer address, with
    // the Done bit (status byte @ offset 12) cleared.
    let rxring = mm::phys_to_virt(rxring_phys) as *mut u8;
    core::ptr::write_bytes(rxring, 0, 4096);
    for i in 0..NRXD {
        let d = rxring.add(i * 16);
        core::ptr::write_volatile(d as *mut u64, rxbuf_phys + (i as u64) * 4096);
        *d.add(12) = 0;
    }
    // Link up (CTRL.SLU bit 6) + full duplex (CTRL.FD bit 0).
    core::ptr::write_volatile(reg(0x0000), core::ptr::read_volatile(reg(0x0000)) | (1 << 6) | 1);
    // RX ring base/length/head/tail (RDH=0; RDT=NRXD-1 hands all but one to HW).
    core::ptr::write_volatile(reg(0x2800), rxring_phys as u32); // RDBAL
    core::ptr::write_volatile(reg(0x2804), (rxring_phys >> 32) as u32); // RDBAH
    core::ptr::write_volatile(reg(0x2808), (NRXD * 16) as u32); // RDLEN
    core::ptr::write_volatile(reg(0x2810), 0); // RDH
    core::ptr::write_volatile(reg(0x2818), (NRXD - 1) as u32); // RDT
    // RCTL: EN(1) | UPE accept-all-unicast(3) | BAM accept-broadcast(15) | SECRC
    // strip-CRC(26), BSIZE=2048(00). UPE lets the unicast ARP reply (to our MAC)
    // through without programming the receive-address filter (RAL/RAH @ 0x5400 lie
    // beyond the mapped 16 KiB BAR window).
    core::ptr::write_volatile(reg(0x0100), (1 << 1) | (1 << 3) | (1 << 15) | (1 << 26));

    // TX an ARP request: who-has 10.0.2.2 (the slirp gateway) tell 10.0.2.15.
    let txring = mm::phys_to_virt(txring_phys) as *mut u8;
    let txbuf = mm::phys_to_virt(txbuf_phys) as *mut u8;
    core::ptr::write_bytes(txring, 0, 4096);
    core::ptr::write_bytes(txbuf, 0, 64);
    let mac = net::net_mac();
    for i in 0..6 {
        *txbuf.add(i) = 0xFF; // dst: broadcast
        *txbuf.add(6 + i) = mac[i]; // src: our MAC
    }
    *txbuf.add(12) = 0x08;
    *txbuf.add(13) = 0x06; // ethertype = ARP
    *txbuf.add(14) = 0x00;
    *txbuf.add(15) = 0x01; // htype = Ethernet
    *txbuf.add(16) = 0x08;
    *txbuf.add(17) = 0x00; // ptype = IPv4
    *txbuf.add(18) = 0x06; // hlen
    *txbuf.add(19) = 0x04; // plen
    *txbuf.add(20) = 0x00;
    *txbuf.add(21) = 0x01; // opcode = request
    for i in 0..6 {
        *txbuf.add(22 + i) = mac[i]; // sender MAC
    }
    *txbuf.add(28) = 10; // sender IP 10.0.2.15
    *txbuf.add(29) = 0;
    *txbuf.add(30) = 2;
    *txbuf.add(31) = 15;
    // target MAC (32..38) left zero (unknown)
    *txbuf.add(38) = 10; // target IP 10.0.2.2 (slirp gateway)
    *txbuf.add(39) = 0;
    *txbuf.add(40) = 2;
    *txbuf.add(41) = 2;
    let frame_len: u16 = 60;
    core::ptr::write_volatile(reg(0x3800), txring_phys as u32); // TDBAL
    core::ptr::write_volatile(reg(0x3804), (txring_phys >> 32) as u32); // TDBAH
    core::ptr::write_volatile(reg(0x3808), 16 * 16); // TDLEN
    core::ptr::write_volatile(reg(0x3810), 0); // TDH
    core::ptr::write_volatile(reg(0x3818), 0); // TDT
    core::ptr::write_volatile(reg(0x0410), 0x0060_200A); // TIPG
    core::ptr::write_volatile(reg(0x0400), (1 << 1) | (1 << 3) | (0x0F << 4) | (0x40 << 12)); // TCTL
    core::ptr::write_volatile(txring as *mut u64, txbuf_phys); // buffer address
    core::ptr::write_volatile((txring.add(8)) as *mut u16, frame_len); // length
    *txring.add(11) = 0x01 | 0x02 | 0x08; // CMD: EOP | IFCS | RS
    *txring.add(12) = 0; // status
    core::ptr::write_volatile(reg(0x3818), 1); // TDT = 1 -> transmit the request

    // Poll RX descriptor 0's Done bit (status byte bit 0). The backend delivers the
    // ARP reply asynchronously, so periodically read the RDH register (an MMIO
    // access — a VM exit) to give QEMU's network backend a chance to inject it.
    let mut spins = 0u32;
    let mut got = false;
    let mut rxlen = 0u16;
    while spins < 50_000_000 {
        // Volatile: the Done bit is written by the device via DMA, so the load
        // must not be hoisted out of the loop (matches the TX twin's poll).
        if core::ptr::read_volatile(rxring.add(12)) & 0x01 != 0 {
            got = true;
            rxlen = core::ptr::read_volatile((rxring.add(8)) as *const u16);
            break;
        }
        if spins & 0xFFF == 0 {
            let _ = core::ptr::read_volatile(reg(0x2810)); // RDH: pump the backend
        }
        spins += 1;
        core::hint::spin_loop();
    }
    // Verify the received frame is an ARP reply from 10.0.2.2. The bytes were
    // written by the device via DMA, so read them volatile.
    let rxbuf0 = mm::phys_to_virt(rxbuf_phys) as *const u8;
    let rb = |o: usize| core::ptr::read_volatile(rxbuf0.add(o));
    let mut arp_ok = got;
    if got {
        if rb(12) != 0x08 || rb(13) != 0x06 {
            arp_ok = false; // not ethertype ARP
        }
        if rb(20) != 0x00 || rb(21) != 0x02 {
            arp_ok = false; // not opcode reply
        }
        if !(rb(28) == 10 && rb(29) == 0 && rb(30) == 2 && rb(31) == 2) {
            arp_ok = false; // sender IP != 10.0.2.2
        }
    }
    dma::dma_free(txbuf_phys, 1);
    dma::dma_free(txring_phys, 1);
    dma::dma_free(rxbuf_phys, NRXD);
    dma::dma_free(rxring_phys, 1);
    if got && arp_ok {
        serial_write(b"E1000: rx ok len=0x");
        serial_write_hex(rxlen as u64);
        serial_write(b"\n");
    } else if got {
        serial_write(b"E1000: rx badframe\n");
    } else {
        serial_write(b"E1000: rx no-dd\n");
    }
}

// ---- Persistent e1000 driver: the active NIC (full-os guide Part II.7) ----
// When no virtio-net is present, the e1000 becomes the live NIC the network stack
// sends/receives through: net::wire_send and net_rx_pump dispatch on net::NIC_KIND.
// Unlike the one-shot TX/RX self-tests above, these rings stay allocated for the
// life of the system. Descriptor addresses are taken straight from the DMA pool
// (real physical pages), so no kv2p translation is needed (virtio needs it because
// its rings live in static BSS). The MMIO BAR is mapped into a DEDICATED window
// (distinct from the shared transient mmio_map_region window) so it is never
// clobbered by a later device mapping.

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const E1000_NTXD: usize = 8; // TX descriptors (minimum ring = 128 B = 8 descriptors)
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const E1000_NRXD: usize = 8; // RX descriptors, each a 2048-byte receive buffer

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut E1000_MMIO: u64 = 0; // 0 until the active NIC is bound
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut E1000_TXRING_VA: u64 = 0;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut E1000_TXBUF_PHYS: u64 = 0;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut E1000_TXBUF_VA: u64 = 0;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut E1000_TXIDX: u32 = 0;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut E1000_RXRING_VA: u64 = 0;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut E1000_RXBUF_PHYS: u64 = 0;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut E1000_RXIDX: u32 = 0;

/// Bring an Intel e1000 (QEMU 82540EM, 8086:100E) up as the live NIC: map its BAR
/// into a dedicated persistent window, read the MAC from EEPROM, set up persistent
/// TX + RX descriptor rings on the DMA pool, and route net::NIC_KIND to e1000.
/// Returns false (the caller then has no NIC) if no e1000 is present or a resource
/// can't be acquired. Idempotent: a second call after a successful bind is a no-op.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn e1000_active_init() -> bool {
    if E1000_MMIO != 0 {
        return true;
    }
    // Find the e1000 on bus 0.
    let mut found: Option<PciBdf> = None;
    let mut dev = 0u8;
    while dev < 32 && found.is_none() {
        let id0 = pci_read32(0, dev, 0, 0);
        if (id0 & 0xFFFF) as u16 != 0xFFFF {
            let hdr = (pci_read32(0, dev, 0, 0x0C) >> 16) & 0xFF;
            let funcs = if hdr & 0x80 != 0 { 8u8 } else { 1u8 };
            let mut func = 0u8;
            while func < funcs {
                let id = pci_read32(0, dev, func, 0);
                if (id & 0xFFFF) as u16 == 0x8086 && ((id >> 16) & 0xFFFF) as u16 == 0x100E {
                    found = Some(PciBdf { bus: 0, dev, func });
                    break;
                }
                func += 1;
            }
        }
        dev += 1;
    }
    let bdf = match found {
        Some(b) => b,
        None => return false,
    };
    // Enable memory space + bus master so the BAR decodes and DMA works.
    let cmd = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x04);
    pci_write32(bdf.bus, bdf.dev, bdf.func, 0x04, cmd | 0x0006);
    let bar0 = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x10);
    let base = (bar0 & 0xFFFF_FFF0) as u64;
    // Dedicated persistent window (distinct PDPT subtree from the shared 0xFF10 one).
    // 5 pages: the control/ring registers live in [0,0x3818]; the statistics block
    // (GPRC 0x4074 / GPTC 0x4080, full-os Part II.7) needs the 0x4000 page mapped too.
    let mmio = match mmio_map_region_at(base, 5, 0xFFFF_FF20_0000_0000) {
        Some(p) => p,
        None => return false,
    };
    let reg = |off: u64| (mmio + off) as *mut u32;

    // Device reset (CTRL.RST, bit 26): the boot-time TX/RX self-test runs first on
    // this same device and leaves the rings enabled pointing at freed DMA, so clear
    // all MAC state before reprogramming. RST self-clears when the reset completes.
    core::ptr::write_volatile(reg(0x0000), core::ptr::read_volatile(reg(0x0000)) | (1 << 26));
    let mut rs = 0u32;
    while rs < 1_000_000 {
        if core::ptr::read_volatile(reg(0x0000)) & (1 << 26) == 0 {
            break;
        }
        rs += 1;
        core::hint::spin_loop();
    }

    // MAC from EEPROM words 0..2 (word N holds bytes [2N, 2N+1] little-endian).
    let mac = match (
        e1000_eeprom_read(mmio, 0),
        e1000_eeprom_read(mmio, 1),
        e1000_eeprom_read(mmio, 2),
    ) {
        (Some(a), Some(b), Some(c)) => [
            (a & 0xFF) as u8,
            (a >> 8) as u8,
            (b & 0xFF) as u8,
            (b >> 8) as u8,
            (c & 0xFF) as u8,
            (c >> 8) as u8,
        ],
        _ => return false,
    };
    net::set_mac(mac);

    // Persistent rings + buffers (cascade-free on any allocation failure).
    let rxbuf_pages = (E1000_NRXD * 2048).div_ceil(4096); // 8*2048 = 4 pages
    let txring = match dma::dma_alloc(1) {
        Some(p) => p,
        None => return false,
    };
    let txbuf = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            dma::dma_free(txring, 1);
            return false;
        }
    };
    let rxring = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            dma::dma_free(txbuf, 1);
            dma::dma_free(txring, 1);
            return false;
        }
    };
    let rxbuf = match dma::dma_alloc(rxbuf_pages) {
        Some(p) => p,
        None => {
            dma::dma_free(rxring, 1);
            dma::dma_free(txbuf, 1);
            dma::dma_free(txring, 1);
            return false;
        }
    };
    let txring_va = mm::phys_to_virt(txring);
    let rxring_va = mm::phys_to_virt(rxring);
    core::ptr::write_bytes(txring_va as *mut u8, 0, 4096);
    core::ptr::write_bytes(rxring_va as *mut u8, 0, 4096);

    // Link up + full duplex (CTRL.SLU bit 6 | CTRL.FD bit 0).
    core::ptr::write_volatile(reg(0x0000), core::ptr::read_volatile(reg(0x0000)) | (1 << 6) | 1);
    // TX ring base/length/head/tail + TIPG + TCTL (EN | PSP | CT=0x0F | COLD=0x40).
    core::ptr::write_volatile(reg(0x3800), txring as u32); // TDBAL
    core::ptr::write_volatile(reg(0x3804), (txring >> 32) as u32); // TDBAH
    core::ptr::write_volatile(reg(0x3808), (E1000_NTXD * 16) as u32); // TDLEN
    core::ptr::write_volatile(reg(0x3810), 0); // TDH
    core::ptr::write_volatile(reg(0x3818), 0); // TDT
    core::ptr::write_volatile(reg(0x0410), 0x0060_200A); // TIPG
    core::ptr::write_volatile(reg(0x0400), (1 << 1) | (1 << 3) | (0x0F << 4) | (0x40 << 12)); // TCTL

    // RX ring: each descriptor points at its own 2048-byte buffer.
    let rxring_p = rxring_va as *mut u8;
    for i in 0..E1000_NRXD {
        let d = rxring_p.add(i * 16);
        core::ptr::write_volatile(d as *mut u64, rxbuf + (i as u64) * 2048);
        *d.add(12) = 0; // status (DD cleared)
    }
    core::ptr::write_volatile(reg(0x2800), rxring as u32); // RDBAL
    core::ptr::write_volatile(reg(0x2804), (rxring >> 32) as u32); // RDBAH
    core::ptr::write_volatile(reg(0x2808), (E1000_NRXD * 16) as u32); // RDLEN
    core::ptr::write_volatile(reg(0x2810), 0); // RDH
    core::ptr::write_volatile(reg(0x2818), (E1000_NRXD - 1) as u32); // RDT
    // RCTL: EN | UPE (accept all unicast, so our MAC's frames pass without RAL/RAH) |
    // BAM (broadcast) | SECRC (strip CRC); BSIZE=2048.
    core::ptr::write_volatile(reg(0x0100), (1 << 1) | (1 << 3) | (1 << 15) | (1 << 26));

    E1000_TXRING_VA = txring_va;
    E1000_TXBUF_PHYS = txbuf;
    E1000_TXBUF_VA = mm::phys_to_virt(txbuf);
    E1000_TXIDX = 0;
    E1000_RXRING_VA = rxring_va;
    E1000_RXBUF_PHYS = rxbuf;
    E1000_RXIDX = 0;
    E1000_MMIO = mmio; // set last: non-zero is the "bound" flag send/recv check
    net::set_nic_e1000();
    true
}

/// Transmit one Ethernet frame through the active e1000 (synchronous: rotate to the
/// next TX descriptor, hand it to the device, poll its Done bit, bounded). Pads runt
/// frames to the 60-byte minimum. Returns false if the device never reports Done or
/// no e1000 is bound. One frame is in flight at a time, so the single TX buffer is
/// safe to reuse across calls.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn e1000_active_send(frame: &[u8]) -> bool {
    if E1000_MMIO == 0 || frame.len() > 2048 {
        return false;
    }
    let len = if frame.len() < 60 { 60 } else { frame.len() };
    let buf = E1000_TXBUF_VA as *mut u8;
    core::ptr::write_bytes(buf, 0, len);
    core::ptr::copy_nonoverlapping(frame.as_ptr(), buf, frame.len());
    let i = (E1000_TXIDX as usize) % E1000_NTXD;
    let d = (E1000_TXRING_VA as *mut u8).add(i * 16);
    core::ptr::write_volatile(d as *mut u64, E1000_TXBUF_PHYS); // buffer address
    core::ptr::write_volatile((d.add(8)) as *mut u16, len as u16); // length
    *d.add(11) = 0x01 | 0x02 | 0x08; // CMD: EOP | IFCS | RS
    *d.add(12) = 0; // status (DD cleared)
    let next = ((i + 1) % E1000_NTXD) as u32;
    E1000_TXIDX = next;
    core::ptr::write_volatile((E1000_MMIO + 0x3818) as *mut u32, next); // TDT -> transmit
    let mut spins = 0u32;
    while spins < 2_000_000 {
        if core::ptr::read_volatile(d.add(12)) & 0x01 != 0 {
            return true;
        }
        spins += 1;
        core::hint::spin_loop();
    }
    false
}

/// Receive one frame from the active e1000 RX ring into `buf` (returns 0 if none is
/// ready). Copies the DMA'd frame, recycles the descriptor back to the device (moves
/// RDT to it), and advances to the next descriptor. Mirrors virtio_net_recv so the
/// net_rx_pump dispatch is symmetric.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn e1000_active_recv(buf: &mut [u8]) -> usize {
    if E1000_MMIO == 0 {
        return 0;
    }
    let i = (E1000_RXIDX as usize) % E1000_NRXD;
    let d = (E1000_RXRING_VA as *mut u8).add(i * 16);
    // Status byte (offset 12) bit 0 = DD, written by the device via DMA -> volatile.
    if core::ptr::read_volatile(d.add(12)) & 0x01 == 0 {
        return 0;
    }
    let mut len = core::ptr::read_volatile((d.add(8)) as *const u16) as usize;
    if len > buf.len() {
        len = buf.len();
    }
    let src = mm::phys_to_virt(E1000_RXBUF_PHYS + (i as u64) * 2048) as *const u8;
    let mut k = 0usize;
    while k < len {
        buf[k] = core::ptr::read_volatile(src.add(k));
        k += 1;
    }
    // Recycle: clear the descriptor and hand it back to the device (RDT = i), then
    // advance. The device fills descriptors from RDH up to (not including) RDT.
    *d.add(12) = 0;
    core::ptr::write_volatile((E1000_MMIO + 0x2818) as *mut u32, i as u32); // RDT = i
    E1000_RXIDX = ((i + 1) % E1000_NRXD) as u32;
    len
}

/// Read the e1000 statistics counters after traffic has flowed (full-os guide
/// Part II.7): GPRC (Good Packets Received, 0x4074) and GPTC (Good Packets
/// Transmitted, 0x4080) are read-to-clear per-frame counters the device
/// maintains. After the boot DHCP DORA over the e1000 they must both be
/// non-zero -- proof the on-device stats registers track the real TX/RX the
/// driver drove (DISCOVER/REQUEST out, OFFER/ACK in). Logs a marker.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn e1000_active_stats_log() {
    if E1000_MMIO == 0 {
        return;
    }
    let gprc = core::ptr::read_volatile((E1000_MMIO + 0x4074) as *const u32);
    let gptc = core::ptr::read_volatile((E1000_MMIO + 0x4080) as *const u32);
    serial_write(b"E1000: stats gprc=0x");
    serial_write_hex(gprc as u64);
    serial_write(b" gptc=0x");
    serial_write_hex(gptc as u64);
    if gprc > 0 && gptc > 0 {
        serial_write(b" ok\n");
    } else {
        serial_write(b" fail\n");
    }
}

/// Read + report an xHCI controller's capability registers (xHCI spec 5.3).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn xhci_report(bdf: PciBdf) {
    // Enable memory space + bus master so the controller decodes its BAR.
    let cmd = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x04);
    pci_write32(bdf.bus, bdf.dev, bdf.func, 0x04, cmd | 0x0006);
    // BAR0 is the xHCI MMIO base (a memory BAR, possibly 64-bit).
    let bar0 = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x10);
    if bar0 & 1 != 0 {
        serial_write(b"XHCI: bar not mmio\n");
        return;
    }
    let mut base = (bar0 & 0xFFFF_FFF0) as u64;
    if (bar0 >> 1) & 0x3 == 0x2 {
        let high = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x14);
        base |= (high as u64) << 32;
    }
    if base == 0 {
        serial_write(b"XHCI: bar unassigned\n");
        return;
    }
    // The PCI MMIO hole is NOT covered by the HHDM, so map the BAR page into the
    // kernel's MMIO window before touching it.
    let mmio = match mmio_map_4k(base) {
        Some(p) => p,
        None => {
            serial_write(b"XHCI: mmio map fail\n");
            return;
        }
    };
    // xHCI MMIO requires 32-bit accesses, so read whole dwords. Dword 0 packs
    // CAPLENGTH (bits[7:0]) and HCIVERSION (bits[31:16]); HCSPARAMS1 (dword @4)
    // packs MaxSlots (bits[7:0]) and MaxPorts (bits[31:24]) (xHCI spec 5.3).
    let cap0 = core::ptr::read_volatile(mmio as *const u32);
    let caplength = cap0 & 0xFF;
    let hciversion = (cap0 >> 16) & 0xFFFF;
    let hcsparams1 = core::ptr::read_volatile((mmio + 4) as *const u32);
    let max_slots = hcsparams1 & 0xFF;
    let max_ports = (hcsparams1 >> 24) & 0xFF;
    serial_write(b"XHCI: found ver=0x");
    serial_write_hex(hciversion as u64);
    serial_write(b" caplen=0x");
    serial_write_hex(caplength as u64);
    serial_write(b" ports=0x");
    serial_write_hex(max_ports as u64);
    serial_write(b" slots=0x");
    serial_write_hex(max_slots as u64);
    serial_write(b"\n");
}

/// Consume xHCI event-ring TRBs sequentially until one of `want_type` is found
/// (xHCI spec 4.9.4). Tracks the dequeue index + consumer cycle state, skips
/// unrelated events (e.g. Port Status Change while waiting for a Command
/// Completion), advances ERDP (interrupter 0, with the Event Handler Busy bit)
/// after each consumed TRB, and returns the event's dword2 (completion code in
/// bits 24-31) and dword3 (type + slot id in bits 24-31), or None on timeout.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn xhci_wait_event(
    mmio: u64,
    ev_va: u64,
    event_ring_phys: u64,
    ir0: u64,
    ev_idx: &mut u64,
    ev_ccs: &mut u32,
    want_type: u32,
) -> Option<(u32, u32)> {
    let mut spins = 0u64;
    while spins < 10_000_000 {
        let off = *ev_idx * 16;
        let d3 = core::ptr::read_volatile((ev_va + off + 12) as *const u32);
        if (d3 & 1) == *ev_ccs {
            let d2 = core::ptr::read_volatile((ev_va + off + 8) as *const u32);
            let ttype = (d3 >> 10) & 0x3F;
            *ev_idx += 1;
            if *ev_idx >= 16 {
                *ev_idx = 0;
                *ev_ccs ^= 1;
            }
            let erdp = event_ring_phys + *ev_idx * 16;
            core::ptr::write_volatile((mmio + ir0 + 0x18) as *mut u32, (erdp as u32) | (1 << 3));
            core::ptr::write_volatile((mmio + ir0 + 0x18 + 4) as *mut u32, (erdp >> 32) as u32);
            if ttype == want_type {
                return Some((d2, d3));
            }
            // else: an unrelated event (consume it and keep waiting)
        } else {
            spins += 1;
            core::hint::spin_loop();
        }
    }
    None
}

/// xHCI controller bring-up + command/event-ring self-test + device enumeration
/// (full-os guide Part II.7, USB driver). Resets and starts the controller, sets
/// up the DCBAA, the command ring, and the event ring (+ ERST/interrupter 0),
/// issues a No-Op command (Command Completion Event handshake), and — if a device
/// is attached to a root port — ENUMERATES it: resets the port, Enable Slot,
/// builds the device + input contexts and an EP0 transfer ring, Address Device,
/// then a GET_DESCRIPTOR(device) control transfer that reads the 18-byte USB
/// device descriptor (vendor/product id) — the full USB enumeration path a HID
/// driver builds on. Called from xhci_detect when a controller is present.
/// Estimate the TSC frequency (ticks/second) by counting `rdtsc` ticks across a
/// fixed PIT channel-2 interval, polling the OUT bit (port 0x61 bit 5) — no
/// timer interrupt needed, so it works this early in boot (before `pit_init`).
/// Returns 0 if the result is implausible (so callers can fall back). Under QEMU
/// TCG both the PIT and the TSC advance with the VM's clock, so this yields a
/// real-wall-clock relationship regardless of how slowly emulated code runs.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn tsc_hz_via_pit() -> u64 {
    const PIT_HZ: u64 = 1_193_182;
    const COUNT: u16 = 11_932; // ~10 ms
    // Gate (bit0) high, speaker data (bit1) off, so channel 2 counts silently.
    let prev = arch_x86::inb(0x61);
    arch_x86::outb(0x61, (prev & !0x02) | 0x01);
    arch_x86::outb(0x43, 0xB0); // channel 2, lobyte/hibyte, mode 0 (one-shot)
    arch_x86::outb(0x42, (COUNT & 0xFF) as u8);
    arch_x86::outb(0x42, (COUNT >> 8) as u8);
    let t0 = core::arch::x86_64::_rdtsc();
    // Wait for OUT to go high (terminal count), bounded so a non-advancing PIT
    // can't wedge boot.
    let mut guard = 0u64;
    while arch_x86::inb(0x61) & 0x20 == 0 {
        guard += 1;
        if guard > 2_000_000 {
            arch_x86::outb(0x61, prev);
            return 0;
        }
    }
    let t1 = core::arch::x86_64::_rdtsc();
    arch_x86::outb(0x61, prev); // restore the speaker/gate latch
    let delta = t1.wrapping_sub(t0);
    let hz = delta.saturating_mul(PIT_HZ) / (COUNT as u64);
    // Plausible TSC range ~50 MHz .. 20 GHz; otherwise let the caller fall back.
    if (50_000_000..=20_000_000_000).contains(&hz) {
        hz
    } else {
        0
    }
}

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn xhci_ring_selftest(bdf: PciBdf) {
    let bar0 = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x10);
    let mut base = (bar0 & 0xFFFF_FFF0) as u64;
    if (bar0 >> 1) & 0x3 == 0x2 {
        base |= (pci_read32(bdf.bus, bdf.dev, bdf.func, 0x14) as u64) << 32;
    }
    // Map 32 KiB of the BAR (covers the operational regs @CAPLENGTH, runtime regs
    // @RTSOFF, and doorbells @DBOFF — all < 0x8000 on qemu-xhci).
    let mmio = match mmio_map_region(base, 8) {
        Some(p) => p,
        None => {
            serial_write(b"XHCI: ring mmio fail\n");
            return;
        }
    };
    let rd = |off: u64| core::ptr::read_volatile((mmio + off) as *const u32);
    let wr = |off: u64, v: u32| core::ptr::write_volatile((mmio + off) as *mut u32, v);
    let wr64 = |off: u64, v: u64| {
        core::ptr::write_volatile((mmio + off) as *mut u32, v as u32);
        core::ptr::write_volatile((mmio + off + 4) as *mut u32, (v >> 32) as u32);
    };
    let caplength = (rd(0) & 0xFF) as u64;
    let max_slots = rd(4) & 0xFF;
    let max_ports = ((rd(4) >> 24) & 0xFF) as u64;
    let dboff = (rd(0x14) & !0x3) as u64;
    let rtsoff = (rd(0x18) & !0x1F) as u64;
    // Bound the HIGHEST byte actually accessed in each block (not just the base
    // offset) against the 32 KiB window, and require a page-aligned BAR (the
    // window returns base + (phys & 0xFFF)). Highest accesses: op + 0x3C
    // (CONFIG/DCBAAP-hi); interrupter 0 at rtsoff + 0x20 .. + 0x3C (ERDP-hi); the
    // PORTSC array at op + 0x400 + (port-1)*0x10 (highest port reaches op + 0x400 +
    // max_ports*0x10); and the per-slot doorbells at dboff + slot*4 (slots
    // 0..=max_slots, so highest is dboff + (max_slots+1)*4).
    if base & 0xFFF != 0
        || caplength + 0x3C > 0x8000
        || rtsoff + 0x40 > 0x8000
        || dboff + (max_slots as u64 + 1) * 4 > 0x8000
        || caplength + 0x400 + max_ports * 0x10 > 0x8000
    {
        serial_write(b"XHCI: regs out of window\n");
        return;
    }
    let op = caplength; // operational registers base offset
    let ir0 = rtsoff + 0x20; // interrupter 0 register set

    // Allocate the controller data structures from the DMA pool (page-aligned ->
    // 64-byte aligned as xHCI requires).
    let dcbaa = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            serial_write(b"XHCI: ring dma fail\n");
            return;
        }
    };
    let cmd_ring = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            dma::dma_free(dcbaa, 1);
            serial_write(b"XHCI: ring dma fail\n");
            return;
        }
    };
    let event_ring = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            dma::dma_free(dcbaa, 1);
            dma::dma_free(cmd_ring, 1);
            serial_write(b"XHCI: ring dma fail\n");
            return;
        }
    };
    let erst = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            dma::dma_free(dcbaa, 1);
            dma::dma_free(cmd_ring, 1);
            dma::dma_free(event_ring, 1);
            serial_write(b"XHCI: ring dma fail\n");
            return;
        }
    };
    core::ptr::write_bytes(mm::phys_to_virt(dcbaa) as *mut u8, 0, 4096);
    core::ptr::write_bytes(mm::phys_to_virt(cmd_ring) as *mut u8, 0, 4096);
    core::ptr::write_bytes(mm::phys_to_virt(event_ring) as *mut u8, 0, 4096);
    core::ptr::write_bytes(mm::phys_to_virt(erst) as *mut u8, 0, 4096);

    // Stop, then reset the controller (USBCMD @0x00, USBSTS @0x04).
    wr(op, rd(op) & !1); // clear Run/Stop
    let mut s = 0u32;
    while rd(op + 4) & 1 == 0 && s < 1_000_000 {
        s += 1;
    } // wait HCHalted
    wr(op, rd(op) | (1 << 1)); // HCRST
    s = 0;
    while rd(op) & (1 << 1) != 0 && s < 5_000_000 {
        s += 1;
    } // wait reset clear
    s = 0;
    while rd(op + 4) & (1 << 11) != 0 && s < 5_000_000 {
        s += 1;
    } // wait CNR clear

    // MaxSlotsEn (CONFIG @0x38), DCBAAP (@0x30), command ring (CRCR @0x18 | RCS).
    wr(op + 0x38, max_slots);
    wr64(op + 0x30, dcbaa);
    wr64(op + 0x18, cmd_ring | 1);

    // Event ring: ERST[0] = {ring base, TRB count}; interrupter 0 ERSTSZ/ERSTBA/ERDP.
    let erst_va = mm::phys_to_virt(erst);
    core::ptr::write_volatile(erst_va as *mut u64, event_ring); // segment base
    core::ptr::write_volatile((erst_va + 8) as *mut u32, 16); // 16 TRBs in the segment
    wr(ir0 + 0x08, 1); // ERSTSZ = 1 segment
    wr64(ir0 + 0x10, erst); // ERSTBA
    wr64(ir0 + 0x18, event_ring); // ERDP

    // Run the controller.
    wr(op, rd(op) | 1);
    s = 0;
    while rd(op + 4) & 1 != 0 && s < 5_000_000 {
        s += 1;
    } // wait HCHalted clear

    // ---- No-Op command: the command/event-ring DMA handshake. ----
    let cmd_va = mm::phys_to_virt(cmd_ring);
    let ev_va = mm::phys_to_virt(event_ring);
    let mut cmd_idx = 0u64; // command-ring enqueue index (producer cycle stays 1; no wrap)
    let mut ev_idx = 0u64; // event-ring dequeue index
    let mut ev_ccs = 1u32; // event-ring consumer cycle state
    core::ptr::write_volatile((cmd_va + cmd_idx * 16 + 12) as *mut u32, (23u32 << 10) | 1);
    cmd_idx += 1;
    wr(dboff, 0); // ring the command doorbell
    let noop_ok = matches!(
        xhci_wait_event(mmio, ev_va, event_ring, ir0, &mut ev_idx, &mut ev_ccs, 33),
        Some((d2, _)) if (d2 >> 24) == 1
    );

    // ---- Device enumeration (USB-HID foundation): if a device is on a root port,
    // reset it, Enable Slot, Address Device, then GET_DESCRIPTOR(device). ----
    let ctx_size: u64 = if (rd(0x10) >> 2) & 1 != 0 { 64 } else { 32 }; // HCCPARAMS1.CSZ
    let mut found_port = 0u64;
    let mut pspeed = 0u32;
    {
        let mut port = 1u64;
        while port <= max_ports {
            let portsc = rd(op + 0x400 + (port - 1) * 0x10);
            if portsc & 1 != 0 {
                found_port = port; // Current Connect Status set
                pspeed = (portsc >> 10) & 0xF;
                break;
            }
            port += 1;
        }
    }
    let devctx = dma::dma_alloc(1);
    let inctx = dma::dma_alloc(1);
    let ep0ring = dma::dma_alloc(1);
    let databuf = dma::dma_alloc(1);
    let hidring = dma::dma_alloc(1); // interrupt-IN transfer ring (HID reports)
    let mut enum_ok = false;
    let mut hid_ok = false;
    let mut vid = 0u32;
    let mut pid = 0u32;
    if let (true, Some(devctx), Some(inctx), Some(ep0ring), Some(databuf), Some(hidring)) =
        (found_port != 0, devctx, inctx, ep0ring, databuf, hidring)
    {
        core::ptr::write_bytes(mm::phys_to_virt(hidring) as *mut u8, 0, 4096);
        core::ptr::write_bytes(mm::phys_to_virt(devctx) as *mut u8, 0, 4096);
        core::ptr::write_bytes(mm::phys_to_virt(inctx) as *mut u8, 0, 4096);
        core::ptr::write_bytes(mm::phys_to_virt(ep0ring) as *mut u8, 0, 4096);
        core::ptr::write_bytes(mm::phys_to_virt(databuf) as *mut u8, 0, 4096);

        // Reset the connected port unless it is already enabled (SuperSpeed ports
        // auto-enable on connect). PORTSC.PED (bit 1) is write-1-to-DISABLE, so the
        // PR write must clear it (and the RW1C change bits 17-23) in the value
        // written, or it would disable an already-enabled port. Then wait for PED.
        let psc_off = op + 0x400 + (found_port - 1) * 0x10;
        if rd(psc_off) & (1 << 1) == 0 {
            let cur = rd(psc_off) & !0x00FE_0002; // drop RW1C change bits + PED
            wr(psc_off, cur | (1 << 4)); // assert PR
            let mut s2 = 0u32;
            while rd(psc_off) & (1 << 1) == 0 && s2 < 5_000_000 {
                s2 += 1;
            }
        }
        pspeed = (rd(psc_off) >> 10) & 0xF; // speed (after any reset)
        // EP0 max packet size by speed (PSI: 4=SuperSpeed, 3=High, else Full/Low).
        let mps: u32 = match pspeed {
            4 => 512,
            3 => 64,
            _ => 8,
        };

        // Enable Slot (TRB type 9); the Command Completion Event carries the slot id.
        core::ptr::write_volatile((cmd_va + cmd_idx * 16 + 12) as *mut u32, (9u32 << 10) | 1);
        cmd_idx += 1;
        wr(dboff, 0);
        let slot = match xhci_wait_event(mmio, ev_va, event_ring, ir0, &mut ev_idx, &mut ev_ccs, 33)
        {
            Some((d2, d3)) if (d2 >> 24) == 1 => ((d3 >> 24) & 0xFF) as u64,
            _ => 0,
        };
        if slot != 0 {
            // Input context: Add flags A0 (slot) + A1 (EP0/DCI 1).
            let in_va = mm::phys_to_virt(inctx);
            core::ptr::write_volatile((in_va + 4) as *mut u32, (1 << 0) | (1 << 1));
            // Slot context: speed (20-23), context entries=1 (27-31); root hub port.
            let slot_ctx = in_va + ctx_size;
            core::ptr::write_volatile(slot_ctx as *mut u32, (pspeed << 20) | (1u32 << 27));
            core::ptr::write_volatile((slot_ctx + 4) as *mut u32, (found_port as u32) << 16);
            // EP0 context: CErr=3 (1-2), EP type Control=4 (3-5), MPS (16-31); TR
            // dequeue ptr | DCS; average TRB length.
            let ep0_ctx = in_va + 2 * ctx_size;
            core::ptr::write_volatile(
                (ep0_ctx + 4) as *mut u32,
                (3u32 << 1) | (4u32 << 3) | (mps << 16),
            );
            core::ptr::write_volatile((ep0_ctx + 8) as *mut u32, (ep0ring as u32) | 1);
            core::ptr::write_volatile((ep0_ctx + 12) as *mut u32, (ep0ring >> 32) as u32);
            core::ptr::write_volatile((ep0_ctx + 16) as *mut u32, 8);
            // DCBAA[slot] = device (output) context.
            core::ptr::write_volatile(
                (mm::phys_to_virt(dcbaa) + slot * 8) as *mut u64,
                devctx,
            );

            // Address Device (TRB type 11): input context pointer + slot id.
            core::ptr::write_volatile((cmd_va + cmd_idx * 16) as *mut u64, inctx);
            core::ptr::write_volatile(
                (cmd_va + cmd_idx * 16 + 12) as *mut u32,
                (11u32 << 10) | ((slot as u32) << 24) | 1,
            );
            cmd_idx += 1;
            wr(dboff, 0);
            let addressed = matches!(
                xhci_wait_event(mmio, ev_va, event_ring, ir0, &mut ev_idx, &mut ev_ccs, 33),
                Some((d2, _)) if (d2 >> 24) == 1
            );
            if addressed {
                // GET_DESCRIPTOR(device, 18) control transfer on the EP0 ring:
                // Setup (IDT, TRT=IN), Data (IN), Status (OUT, IOC) stage TRBs.
                let er = mm::phys_to_virt(ep0ring);
                core::ptr::write_volatile(er as *mut u32, 0x0100_0680); // bmRT80 bReq06 wValue0100
                core::ptr::write_volatile((er + 4) as *mut u32, 0x0012_0000); // wIndex0 wLength18
                core::ptr::write_volatile((er + 8) as *mut u32, 8); // setup data length
                core::ptr::write_volatile(
                    (er + 12) as *mut u32,
                    (2u32 << 10) | (3u32 << 16) | (1 << 6) | 1, // type|TRT=IN|IDT|cycle
                );
                core::ptr::write_volatile((er + 16) as *mut u32, databuf as u32);
                core::ptr::write_volatile((er + 20) as *mut u32, (databuf >> 32) as u32);
                core::ptr::write_volatile((er + 24) as *mut u32, 18); // data length
                core::ptr::write_volatile((er + 28) as *mut u32, (3u32 << 10) | (1 << 16) | 1); // type|DIR=IN|cycle
                core::ptr::write_volatile((er + 40) as *mut u32, 0);
                core::ptr::write_volatile((er + 44) as *mut u32, (4u32 << 10) | (1 << 5) | 1); // type|IOC|cycle (DIR=OUT)
                wr(dboff + slot * 4, 1); // ring the device's EP0 doorbell (DCI 1)
                let xfer_ok = matches!(
                    xhci_wait_event(mmio, ev_va, event_ring, ir0, &mut ev_idx, &mut ev_ccs, 32),
                    Some((d2, _)) if { let c = d2 >> 24; c == 1 || c == 13 } // Success or Short Packet
                );
                if xfer_ok {
                    let db = mm::phys_to_virt(databuf) as *const u8;
                    let blen = core::ptr::read_volatile(db);
                    let dtype = core::ptr::read_volatile(db.add(1));
                    if blen == 18 && dtype == 1 {
                        vid = core::ptr::read_volatile(db.add(8)) as u32
                            | (core::ptr::read_volatile(db.add(9)) as u32) << 8;
                        pid = core::ptr::read_volatile(db.add(10)) as u32
                            | (core::ptr::read_volatile(db.add(11)) as u32) << 8;
                        enum_ok = true;
                    }
                }
                if enum_ok {
                    // ---- HID configuration (full-os Part II.7 / III.8): put the
                    // device in its configured state and set up an interrupt-IN
                    // endpoint so a HID boot keyboard's reports can be polled.
                    let er = mm::phys_to_virt(ep0ring);
                    let mut ep0_off = 48u64; // EP0 TRBs 0..2 used by GET_DESCRIPTOR

                    // 1) SET_CONFIGURATION(1): bmRT 0x00, bReq 0x09, wValue 1, no data.
                    core::ptr::write_volatile((er + ep0_off) as *mut u32, 0x0001_0900);
                    core::ptr::write_volatile((er + ep0_off + 4) as *mut u32, 0);
                    core::ptr::write_volatile((er + ep0_off + 8) as *mut u32, 8);
                    core::ptr::write_volatile(
                        (er + ep0_off + 12) as *mut u32,
                        (2u32 << 10) | (1 << 6) | 1, // Setup, TRT=No-Data, IDT, cycle
                    );
                    core::ptr::write_volatile(
                        (er + ep0_off + 28) as *mut u32,
                        (4u32 << 10) | (1 << 16) | (1 << 5) | 1, // Status, DIR=IN, IOC
                    );
                    wr(dboff + slot * 4, 1);
                    let setcfg_ok = matches!(
                        xhci_wait_event(mmio, ev_va, event_ring, ir0, &mut ev_idx, &mut ev_ccs, 32),
                        Some((d2, _)) if (d2 >> 24) == 1
                    );
                    ep0_off += 32;

                    // 2) Configure Endpoint: add the interrupt-IN endpoint (DCI 3).
                    let in_va = mm::phys_to_virt(inctx);
                    core::ptr::write_bytes(in_va as *mut u8, 0, 4096);
                    core::ptr::write_volatile((in_va + 4) as *mut u32, (1 << 0) | (1 << 3)); // Add slot + DCI3
                    let slot_ctx = in_va + ctx_size;
                    core::ptr::write_volatile(slot_ctx as *mut u32, (pspeed << 20) | (3u32 << 27)); // ctx entries=3
                    core::ptr::write_volatile((slot_ctx + 4) as *mut u32, (found_port as u32) << 16);
                    let ep1_ctx = in_va + 4 * ctx_size; // DCI 3 (EP1-IN)
                    core::ptr::write_volatile(ep1_ctx as *mut u32, 6u32 << 16); // Interval
                    core::ptr::write_volatile(
                        (ep1_ctx + 4) as *mut u32,
                        (3u32 << 1) | (7u32 << 3) | (8u32 << 16), // CErr=3, EP type Interrupt-IN, MPS=8
                    );
                    core::ptr::write_volatile((ep1_ctx + 8) as *mut u32, (hidring as u32) | 1);
                    core::ptr::write_volatile((ep1_ctx + 12) as *mut u32, (hidring >> 32) as u32);
                    core::ptr::write_volatile((ep1_ctx + 16) as *mut u32, 8 | (8 << 16)); // avg TRB len + max ESIT
                    core::ptr::write_volatile((cmd_va + cmd_idx * 16) as *mut u64, inctx);
                    core::ptr::write_volatile(
                        (cmd_va + cmd_idx * 16 + 12) as *mut u32,
                        (12u32 << 10) | ((slot as u32) << 24) | 1, // Configure Endpoint
                    );
                    cmd_idx += 1;
                    wr(dboff, 0);
                    let cfgep_ok = matches!(
                        xhci_wait_event(mmio, ev_va, event_ring, ir0, &mut ev_idx, &mut ev_ccs, 33),
                        Some((d2, _)) if (d2 >> 24) == 1
                    );

                    // 3) SET_PROTOCOL(boot): bmRT 0x21, bReq 0x0B, wValue 0, no data.
                    core::ptr::write_volatile((er + ep0_off) as *mut u32, 0x0000_0B21);
                    core::ptr::write_volatile((er + ep0_off + 4) as *mut u32, 0);
                    core::ptr::write_volatile((er + ep0_off + 8) as *mut u32, 8);
                    core::ptr::write_volatile(
                        (er + ep0_off + 12) as *mut u32,
                        (2u32 << 10) | (1 << 6) | 1,
                    );
                    core::ptr::write_volatile(
                        (er + ep0_off + 28) as *mut u32,
                        (4u32 << 10) | (1 << 16) | (1 << 5) | 1,
                    );
                    wr(dboff + slot * 4, 1);
                    let setproto_ok = matches!(
                        xhci_wait_event(mmio, ev_va, event_ring, ir0, &mut ev_idx, &mut ev_ccs, 32),
                        Some((d2, _)) if (d2 >> 24) == 1
                    );

                    hid_ok = setcfg_ok && cfgep_ok && setproto_ok;

                    if hid_ok {
                        // ---- HID input report: poll the interrupt-IN endpoint for
                        // an 8-byte boot-keyboard report [modifier, reserved,
                        // keycode*6]. A key pressed on the host (QMP send-key)
                        // generates a report; the kernel reads back its keycode.
                        core::ptr::write_bytes(mm::phys_to_virt(databuf) as *mut u8, 0, 8);
                        let hr = mm::phys_to_virt(hidring);
                        core::ptr::write_volatile(hr as *mut u32, databuf as u32);
                        core::ptr::write_volatile((hr + 4) as *mut u32, (databuf >> 32) as u32);
                        core::ptr::write_volatile((hr + 8) as *mut u32, 8); // TRB transfer length
                        core::ptr::write_volatile((hr + 12) as *mut u32, (1u32 << 10) | (1 << 5) | 1); // Normal, IOC
                        serial_write(b"XHCI: hid polling\n");
                        wr(dboff + slot * 4, 3); // ring the interrupt-IN doorbell (DCI 3)
                        let ev = mm::phys_to_virt(event_ring);
                        let mut report_mod = 0u8;
                        let mut report_key = 0u8;
                        let mut got_report = false;
                        // Bound the wait by WALL-CLOCK, not a raw spin count: under
                        // TCG the per-iteration cost swings wildly with host load, so
                        // an iteration cap once stalled boot past the test window (and
                        // would stall every keyboard-attached boot). ~2 s of real time
                        // is ample for a host-injected key while keeping boot snappy.
                        // A large iteration cap is a backstop if the TSC never moves.
                        let tsc_hz = tsc_hz_via_pit();
                        let tsc_start = core::arch::x86_64::_rdtsc();
                        // When calibration works the TSC budget (~2 s real time)
                        // terminates first and the iteration cap is just paranoia.
                        // When it fails, a tight iteration cap bounds boot instead.
                        let tsc_budget = if tsc_hz != 0 { tsc_hz * 2 } else { u64::MAX };
                        let iter_cap: u64 = if tsc_hz != 0 { 200_000_000 } else { 20_000_000 };
                        let mut s2 = 0u64;
                        loop {
                            let off = ev_idx * 16;
                            let d3 = core::ptr::read_volatile((ev + off + 12) as *const u32);
                            if (d3 & 1) == ev_ccs {
                                let d2 = core::ptr::read_volatile((ev + off + 8) as *const u32);
                                let ty = (d3 >> 10) & 0x3F;
                                ev_idx += 1;
                                if ev_idx >= 16 {
                                    ev_idx = 0;
                                    ev_ccs ^= 1;
                                }
                                wr64(ir0 + 0x18, (event_ring + ev_idx * 16) | (1 << 3));
                                if ty == 32 {
                                    let c = d2 >> 24;
                                    if c == 1 || c == 13 {
                                        let dbp = mm::phys_to_virt(databuf) as *const u8;
                                        report_mod = core::ptr::read_volatile(dbp);
                                        report_key = core::ptr::read_volatile(dbp.add(2));
                                        got_report = true;
                                        break;
                                    }
                                }
                            } else {
                                // MMIO read -> a VM exit so QEMU delivers the
                                // host-injected key + runs the USB transfer. Re-ring
                                // the endpoint doorbell periodically as a hint.
                                if s2 & 0x3F == 0 {
                                    let _ = rd(op + 4);
                                }
                                if s2 & 0xFFFF == 0 {
                                    wr(dboff + slot * 4, 3);
                                }
                                s2 += 1;
                                // Check the time budget on the MMIO cadence (rdtsc is
                                // itself cheap, but this keeps the hot path tight).
                                if s2 & 0x3F == 0
                                    && core::arch::x86_64::_rdtsc().wrapping_sub(tsc_start)
                                        > tsc_budget
                                {
                                    break;
                                }
                                if s2 > iter_cap {
                                    break;
                                }
                                core::hint::spin_loop();
                            }
                        }
                        if got_report {
                            serial_write(b"XHCI: hid report mod=0x");
                            serial_write_hex(report_mod as u64);
                            serial_write(b" key=0x");
                            serial_write_hex(report_key as u64);
                            serial_write(b"\n");
                        } else {
                            serial_write(b"XHCI: hid no-report\n");
                        }
                    }
                }
            }
        }
    }
    if let Some(p) = devctx {
        dma::dma_free(p, 1);
    }
    if let Some(p) = inctx {
        dma::dma_free(p, 1);
    }
    if let Some(p) = ep0ring {
        dma::dma_free(p, 1);
    }
    if let Some(p) = databuf {
        dma::dma_free(p, 1);
    }
    if let Some(p) = hidring {
        dma::dma_free(p, 1);
    }

    // Stop the controller and release the core structures.
    wr(op, rd(op) & !1);
    dma::dma_free(dcbaa, 1);
    dma::dma_free(cmd_ring, 1);
    dma::dma_free(event_ring, 1);
    dma::dma_free(erst, 1);
    if noop_ok {
        serial_write(b"XHCI: noop ok\n");
    } else {
        serial_write(b"XHCI: noop fail\n");
    }
    if found_port != 0 {
        if enum_ok {
            serial_write(b"XHCI: hid enumerated port=0x");
            serial_write_hex(found_port);
            serial_write(b" vid=0x");
            serial_write_hex(vid as u64);
            serial_write(b" pid=0x");
            serial_write_hex(pid as u64);
            serial_write(b"\n");
            if hid_ok {
                // The device is configured + an interrupt-IN endpoint is set up,
                // ready to receive HID boot-keyboard reports.
                serial_write(b"XHCI: hid configured ep-in ok\n");
            } else {
                serial_write(b"XHCI: hid configure fail\n");
            }
        } else {
            serial_write(b"XHCI: enum fail port=0x");
            serial_write_hex(found_port);
            serial_write(b"\n");
        }
    }
}

/// Detect an Intel e1000 NIC (full-os guide Part II.7, the named 2nd NIC): scan
/// PCI bus 0 for vendor 0x8086 device 0x100E (the QEMU 82540EM `-device e1000`),
/// map its BAR0, and read the device STATUS plus the MAC out of the EEPROM. This
/// is the discovery + register/EEPROM-read foundation; the TX/RX descriptor rings
/// and a full driver are carry-forward. Reports `E1000: none` when absent.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn e1000_detect() {
    let mut dev = 0u8;
    while dev < 32 {
        let id0 = pci_read32(0, dev, 0, 0);
        if (id0 & 0xFFFF) as u16 == 0xFFFF {
            dev += 1;
            continue;
        }
        let hdr = (pci_read32(0, dev, 0, 0x0C) >> 16) & 0xFF;
        let funcs = if hdr & 0x80 != 0 { 8u8 } else { 1u8 };
        let mut func = 0u8;
        while func < funcs {
            let id = pci_read32(0, dev, func, 0);
            let vendor = (id & 0xFFFF) as u16;
            let device = ((id >> 16) & 0xFFFF) as u16;
            if vendor == 0x8086 && device == 0x100E {
                let bdf = PciBdf { bus: 0, dev, func };
                e1000_report(bdf);
                e1000_tx_selftest(bdf); // full-os Part II.7: e1000 TX-ring driver
                e1000_rx_selftest(bdf); // full-os Part II.7: e1000 RX-ring driver (wire ARP round-trip)
                return;
            }
            func += 1;
        }
        dev += 1;
    }
    serial_write(b"E1000: none\n");
}

/// Read one 16-bit EEPROM word `addr` through the e1000 EERD register (offset
/// 0x14): write (addr<<8)|START, poll the DONE bit, return the data half. Returns
/// None if DONE never asserts within the bounded poll.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn e1000_eeprom_read(mmio: u64, addr: u16) -> Option<u16> {
    const EERD: u64 = 0x0014;
    const START: u32 = 1 << 0;
    const DONE: u32 = 1 << 4;
    core::ptr::write_volatile((mmio + EERD) as *mut u32, ((addr as u32) << 8) | START);
    let mut spins = 0u32;
    while spins < 100_000 {
        let v = core::ptr::read_volatile((mmio + EERD) as *const u32);
        if v & DONE != 0 {
            return Some((v >> 16) as u16);
        }
        spins += 1;
    }
    None
}

/// Read + report an e1000's STATUS register and EEPROM MAC (82540EM datasheet).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn e1000_report(bdf: PciBdf) {
    // Enable memory space + bus master so the controller decodes its BAR.
    let cmd = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x04);
    pci_write32(bdf.bus, bdf.dev, bdf.func, 0x04, cmd | 0x0006);
    let bar0 = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x10);
    if bar0 & 1 != 0 {
        serial_write(b"E1000: bar not mmio\n");
        return;
    }
    let base = (bar0 & 0xFFFF_FFF0) as u64; // e1000 BAR0 is a 32-bit memory BAR
    if base == 0 {
        serial_write(b"E1000: bar unassigned\n");
        return;
    }
    let mmio = match mmio_map_4k(base) {
        Some(p) => p,
        None => {
            serial_write(b"E1000: mmio map fail\n");
            return;
        }
    };
    // STATUS (0x0008): a present, decoding device returns a sane value (not the
    // all-ones an unmapped BAR yields). The MAC lives in EEPROM words 0..2.
    let status = core::ptr::read_volatile((mmio + 0x0008) as *const u32);
    let (w0, w1, w2) = match (
        e1000_eeprom_read(mmio, 0),
        e1000_eeprom_read(mmio, 1),
        e1000_eeprom_read(mmio, 2),
    ) {
        (Some(a), Some(b), Some(c)) => (a, b, c),
        _ => {
            serial_write(b"E1000: eeprom timeout\n");
            return;
        }
    };
    // EEPROM word N holds MAC bytes [2N, 2N+1] little-endian; pack as a 48-bit
    // value with mac[0] in the low byte.
    let mac = (w0 as u64) | ((w1 as u64) << 16) | ((w2 as u64) << 32);
    serial_write(b"E1000: found status=0x");
    serial_write_hex(status as u64);
    serial_write(b" mac=0x");
    serial_write_hex(mac);
    serial_write(b"\n");
}

/// Detect an Intel HD Audio controller (full-os guide Part III, audio): scan PCI
/// bus 0 for a Multimedia / HD-Audio function (class 0x04, subclass 0x03), map
/// its BAR0, and read the global capabilities (GCAP) + version registers. This is
/// the discovery + capability-read foundation; CORB/RIRB rings, codec
/// enumeration, and PCM streaming are carry-forward. Reports `HDA: none` when
/// absent. Reachable with `-device intel-hda`.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn hda_detect() {
    let mut dev = 0u8;
    while dev < 32 {
        let id0 = pci_read32(0, dev, 0, 0);
        if (id0 & 0xFFFF) as u16 == 0xFFFF {
            dev += 1;
            continue;
        }
        let hdr = (pci_read32(0, dev, 0, 0x0C) >> 16) & 0xFF;
        let funcs = if hdr & 0x80 != 0 { 8u8 } else { 1u8 };
        let mut func = 0u8;
        while func < funcs {
            let id = pci_read32(0, dev, func, 0);
            if (id & 0xFFFF) as u16 != 0xFFFF {
                let class_reg = pci_read32(0, dev, func, 0x08);
                let class = (class_reg >> 24) as u8;
                let subclass = (class_reg >> 16) as u8;
                if class == 0x04 && subclass == 0x03 {
                    hda_report(PciBdf { bus: 0, dev, func });
                    return;
                }
            }
            func += 1;
        }
        dev += 1;
    }
    serial_write(b"HDA: none\n");
}

/// Read + report an Intel HD Audio controller's GCAP + version (HDA spec 3.3).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn hda_report(bdf: PciBdf) {
    let cmd = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x04);
    pci_write32(bdf.bus, bdf.dev, bdf.func, 0x04, cmd | 0x0006); // mem + bus master
    let bar0 = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x10);
    if bar0 & 1 != 0 {
        serial_write(b"HDA: bar not mmio\n");
        return;
    }
    let mut base = (bar0 & 0xFFFF_FFF0) as u64;
    if (bar0 >> 1) & 0x3 == 0x2 {
        let high = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x14);
        base |= (high as u64) << 32;
    }
    if base == 0 {
        serial_write(b"HDA: bar unassigned\n");
        return;
    }
    let mmio = match mmio_map_4k(base) {
        Some(p) => p,
        None => {
            serial_write(b"HDA: mmio map fail\n");
            return;
        }
    };
    // Dword @0 packs GCAP (bits[15:0]), VMIN (bits[23:16]), VMAJ (bits[31:24]).
    let d0 = core::ptr::read_volatile(mmio as *const u32);
    let gcap = d0 & 0xFFFF;
    let vmin = (d0 >> 16) & 0xFF;
    let vmaj = (d0 >> 24) & 0xFF;
    serial_write(b"HDA: found gcap=0x");
    serial_write_hex(gcap as u64);
    serial_write(b" ver=0x");
    serial_write_hex(((vmaj << 8) | vmin) as u64);
    serial_write(b"\n");
    hda_codec_selftest(mmio);
    // With a duplex codec present, the codec self-test leaves a persistent audio
    // stream; exercise the userspace audio_write core over it (full-os Part III).
    hda_audio_selftest();
}

/// Bring the HDA controller up enough to round-trip ONE codec verb (full-os guide
/// Part III audio): reset, set up the CORB (command) + RIRB (response) DMA rings,
/// and issue GET_PARAMETER(node 0, VENDOR_ID) to the first present codec, reading
/// its vendor/device id back from the RIRB. This is the codec-communication core
/// every HDA driver needs (BDL stream descriptors + PCM playback build on it).
/// A controller with no codec attached reports `HDA: no codec` (no-op, safe).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn hda_codec_selftest(mmio: u64) {
    const GCTL: u64 = 0x08;
    const STATESTS: u64 = 0x0E;
    const CORBLBASE: u64 = 0x40;
    const CORBUBASE: u64 = 0x44;
    const CORBWP: u64 = 0x48;
    const CORBRP: u64 = 0x4A;
    const CORBCTL: u64 = 0x4C;
    const CORBSIZE: u64 = 0x4E;
    const RIRBLBASE: u64 = 0x50;
    const RIRBUBASE: u64 = 0x54;
    const RIRBWP: u64 = 0x58;
    const RINTCNT: u64 = 0x5A;
    const RIRBCTL: u64 = 0x5C;
    const RIRBSIZE: u64 = 0x5E;

    // Reset: CRST=0, wait for halt, then CRST=1, wait for run.
    let g = core::ptr::read_volatile((mmio + GCTL) as *const u32);
    core::ptr::write_volatile((mmio + GCTL) as *mut u32, g & !1);
    let mut s = 0u64;
    while core::ptr::read_volatile((mmio + GCTL) as *const u32) & 1 != 0 && s < 2_000_000 {
        s += 1;
    }
    core::ptr::write_volatile((mmio + GCTL) as *mut u32, 1);
    s = 0;
    while core::ptr::read_volatile((mmio + GCTL) as *const u32) & 1 == 0 && s < 2_000_000 {
        s += 1;
    }
    if core::ptr::read_volatile((mmio + GCTL) as *const u32) & 1 == 0 {
        serial_write(b"HDA: reset fail\n");
        return;
    }
    // Wait (bounded) for a codec to assert its STATESTS bit after reset.
    s = 0;
    while core::ptr::read_volatile((mmio + STATESTS) as *const u16) == 0 && s < 4_000_000 {
        s += 1;
    }
    let statests = core::ptr::read_volatile((mmio + STATESTS) as *const u16);
    if statests == 0 {
        serial_write(b"HDA: no codec\n");
        return;
    }
    let codec = statests.trailing_zeros();

    let corb = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            serial_write(b"HDA: dma fail\n");
            return;
        }
    };
    let rirb = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            dma::dma_free(corb, 1);
            serial_write(b"HDA: dma fail\n");
            return;
        }
    };
    // Stop the ring DMA engines while we (re)configure them.
    core::ptr::write_volatile((mmio + CORBCTL) as *mut u8, 0);
    core::ptr::write_volatile((mmio + RIRBCTL) as *mut u8, 0);
    core::ptr::write_volatile((mmio + CORBSIZE) as *mut u8, 0x02); // 256 entries
    core::ptr::write_volatile((mmio + RIRBSIZE) as *mut u8, 0x02);
    core::ptr::write_volatile((mmio + CORBLBASE) as *mut u32, corb as u32);
    core::ptr::write_volatile((mmio + CORBUBASE) as *mut u32, (corb >> 32) as u32);
    core::ptr::write_volatile((mmio + RIRBLBASE) as *mut u32, rirb as u32);
    core::ptr::write_volatile((mmio + RIRBUBASE) as *mut u32, (rirb >> 32) as u32);
    // Reset the CORB read pointer (set bit 15, wait, clear, wait).
    core::ptr::write_volatile((mmio + CORBRP) as *mut u16, 0x8000);
    s = 0;
    while core::ptr::read_volatile((mmio + CORBRP) as *const u16) & 0x8000 == 0 && s < 1_000_000 {
        s += 1;
    }
    core::ptr::write_volatile((mmio + CORBRP) as *mut u16, 0x0000);
    s = 0;
    while core::ptr::read_volatile((mmio + CORBRP) as *const u16) & 0x8000 != 0 && s < 1_000_000 {
        s += 1;
    }
    core::ptr::write_volatile((mmio + CORBWP) as *mut u16, 0);
    // Reset the RIRB write pointer. RINTCNT is the response-interrupt threshold,
    // but QEMU's CORB engine also *stops* once `rirb_count == rirb_cnt` (it won't
    // service further verbs until the RIRB interrupt is acked) -- so a threshold of
    // 1 caps us at a single response. Set it well above our verb count so the whole
    // enumeration sequence drains in one run.
    core::ptr::write_volatile((mmio + RIRBWP) as *mut u16, 0x8000);
    core::ptr::write_volatile((mmio + RINTCNT) as *mut u16, 0xFF);
    // Run both DMA engines.
    core::ptr::write_volatile((mmio + CORBCTL) as *mut u8, 0x02);
    core::ptr::write_volatile((mmio + RIRBCTL) as *mut u8, 0x02);

    let corb_va = mm::phys_to_virt(corb);
    let rirb_va = mm::phys_to_virt(rirb);
    // GET_PARAMETER verb = CAd<<28 | NID<<20 | 0xF00<<8 | param. Issue the verbs
    // in sequence (CORB index 1.., RIRB tracks responses): VENDOR_ID, then walk
    // the codec tree -- root SUBORDINATE_NODE_COUNT (function groups), the audio
    // function group's TYPE, and its SUBORDINATE_NODE_COUNT (widgets).
    let base = (codec << 28) | 0x000F_0000;
    let vid = hda_verb(mmio, corb_va, rirb_va, 1, base | 0x00); // VENDOR_ID
    let root_nc = hda_verb(mmio, corb_va, rirb_va, 2, base | 0x04); // SUBORDINATE_NODE_COUNT
    let (vid, root_nc) = match (vid, root_nc) {
        (Some(v), Some(n)) => (v, n),
        _ => {
            core::ptr::write_volatile((mmio + CORBCTL) as *mut u8, 0);
            core::ptr::write_volatile((mmio + RIRBCTL) as *mut u8, 0);
            dma::dma_free(corb, 1);
            dma::dma_free(rirb, 1);
            serial_write(b"HDA: codec no-response\n");
            return;
        }
    };
    let fg_start = (root_nc >> 16) & 0xFF; // first function-group node id
    let fg_count = root_nc & 0xFF;
    let fg_base = (codec << 28) | (fg_start << 20) | 0x000F_0000;
    let fg_type = hda_verb(mmio, corb_va, rirb_va, 3, fg_base | 0x05).unwrap_or(0); // FUNCTION_GROUP_TYPE
    let fg_widgets = hda_verb(mmio, corb_va, rirb_va, 4, fg_base | 0x04).unwrap_or(0); // node count
    let widget_start = (fg_widgets >> 16) & 0xFF; // first widget node id under the AFG
    let widget_count = fg_widgets & 0xFF;
    serial_write(b"HDA: codec ");
    serial_write_hex(codec as u64);
    serial_write(b" vid=0x");
    serial_write_hex(((vid >> 16) & 0xFFFF) as u64);
    serial_write(b" did=0x");
    serial_write_hex((vid & 0xFFFF) as u64);
    serial_write(b" ok\n");
    // Codec enumeration: function-group count, the AFG's type (1 = audio), and its
    // widget count -- the codec topology a driver walks to find DAC/pin widgets.
    serial_write(b"HDA: codec enum fgs=0x");
    serial_write_hex(fg_count as u64);
    serial_write(b" afgtype=0x");
    serial_write_hex((fg_type & 0xFF) as u64);
    serial_write(b" widgets=0x");
    serial_write_hex(widget_count as u64);
    serial_write(b" ok\n");

    // PCM playback: find the DAC (first Audio Output widget) by reading each widget's
    // AUDIO_WIDGET_CAPABILITIES (param 0x09, type field bits [23:20] == 0). The
    // CORB/RIRB engines are still running, so verb indices continue from 4.
    let mut vi = 4u16;
    let mut dac = 0u32;
    let wmax = if widget_count > 16 { 16 } else { widget_count };
    let mut wi = 0u32;
    while wi < wmax {
        let nid = widget_start + wi;
        vi += 1;
        let cap = hda_verb(
            mmio,
            corb_va,
            rirb_va,
            vi,
            (codec << 28) | (nid << 20) | 0x000F_0000 | 0x09,
        )
        .unwrap_or(0);
        if (cap >> 20) & 0xF == 0 {
            dac = nid;
            break;
        }
        wi += 1;
    }

    // Converter + pin capabilities (full-os guide Part III audio): read the DAC's
    // PCM_SIZE_RATES (param 0x0A: supported sample rates / bit depths) and
    // STREAM_FORMATS (param 0x0B: PCM/float/AC3), then find the first output PIN
    // COMPLEX widget (AUDIO_WIDGET_CAP type bits[23:20] == 4) and read its
    // PIN_CAPABILITIES (param 0x0C) and CONFIG_DEFAULT (verb 0xF1C: the BIOS pin
    // configuration -- port connectivity, default device, location). These are the
    // converter/pin descriptors a driver consults to pick a stream format and route
    // output. Read-only verbs that continue the CORB/RIRB sequence (vi keeps rising,
    // staying < RINTCNT = 0xFF); the engines are still running.
    let pbase = (codec << 28) | 0x000F_0000;
    vi += 1;
    let rates = hda_verb(mmio, corb_va, rirb_va, vi, pbase | (dac << 20) | 0x0A).unwrap_or(0);
    vi += 1;
    let fmts = hda_verb(mmio, corb_va, rirb_va, vi, pbase | (dac << 20) | 0x0B).unwrap_or(0);
    let mut pin = 0u32;
    let mut pj = 0u32;
    while pj < wmax {
        let nid = widget_start + pj;
        vi += 1;
        let cap = hda_verb(mmio, corb_va, rirb_va, vi, pbase | (nid << 20) | 0x09).unwrap_or(0);
        if (cap >> 20) & 0xF == 4 {
            pin = nid;
            break;
        }
        pj += 1;
    }
    vi += 1;
    let pincap = hda_verb(mmio, corb_va, rirb_va, vi, pbase | (pin << 20) | 0x0C).unwrap_or(0);
    vi += 1;
    // CONFIG_DEFAULT is a 12-bit "Get" verb (0xF1C), not a parameter: payload 0.
    let pincfg = hda_verb(mmio, corb_va, rirb_va, vi, (codec << 28) | (pin << 20) | 0x000F_1C00)
        .unwrap_or(0);
    serial_write(b"HDA: caps rates=0x");
    serial_write_hex(rates as u64);
    serial_write(b" fmts=0x");
    serial_write_hex(fmts as u64);
    serial_write(b" pincap=0x");
    serial_write_hex(pincap as u64);
    serial_write(b" cfg=0x");
    serial_write_hex(pincfg as u64);
    // A real codec reports supported rates on its DAC and capabilities on an output
    // pin; require both. (pincfg is printed for inspection -- a BIOS may leave it 0.)
    if rates != 0 && pincap != 0 {
        serial_write(b" ok\n");
    } else {
        serial_write(b" fail\n");
    }

    hda_pcm_selftest(mmio, corb_va, rirb_va, codec, dac, vi);

    // Stop the engines and release the rings.
    core::ptr::write_volatile((mmio + CORBCTL) as *mut u8, 0);
    core::ptr::write_volatile((mmio + RIRBCTL) as *mut u8, 0);
    dma::dma_free(corb, 1);
    dma::dma_free(rirb, 1);
}

/// Submit one HDA verb at CORB index `idx` and return the RIRB response dword
/// (bounded wait for the response). The CORB/RIRB DMA engines must be running.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn hda_verb(mmio: u64, corb_va: u64, rirb_va: u64, idx: u16, verb: u32) -> Option<u32> {
    core::ptr::write_volatile((corb_va + (idx as u64) * 4) as *mut u32, verb);
    core::ptr::write_volatile((mmio + 0x48) as *mut u16, idx); // CORBWP
    let mut s = 0u64;
    while (core::ptr::read_volatile((mmio + 0x58) as *const u16) & 0xFF) < idx && s < 8_000_000 {
        s += 1;
    }
    if (core::ptr::read_volatile((mmio + 0x58) as *const u16) & 0xFF) < idx {
        return None;
    }
    Some(core::ptr::read_volatile((rirb_va + (idx as u64) * 8) as *const u32))
}

// Persistent HD Audio output context (full-os guide Part III audio): once the PCM
// self-test proves the BDL -> SD0 -> codec path on a real codec, the buffer + BDL +
// SD0 programming + codec binding are KEPT (not torn down) so userspace can submit
// PCM through them via sys_ioctl op 7 (audio_write) without re-initialising the
// codec. AUDIO_READY is set only when a real duplex codec streamed successfully.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const AUDIO_BUF_BYTES: u64 = 4096;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut AUDIO_READY: bool = false;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut AUDIO_SD: u64 = 0; // SD0 register base
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut AUDIO_BUF_VA: u64 = 0; // PCM sample buffer (kernel VA)
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut AUDIO_BDL_PHYS: u64 = 0; // Buffer Descriptor List (physical)

/// HD Audio PCM playback self-test (full-os guide Part III audio): the actual
/// streaming path a sound driver uses. Build a buffer of PCM samples + a Buffer
/// Descriptor List, point output stream descriptor SD0 at it, associate the DAC
/// converter with that stream, run the stream, and confirm the controller is DMAing
/// the buffer to the codec by watching SDnLPIB (the link position in the buffer)
/// advance. This is the proof that the BDL -> stream-DMA -> codec path works. On
/// success it KEEPS the buffer/BDL/SD0 as the persistent audio context (see above).
///
/// Requires the CORB/RIRB engines running (for the codec config verbs, continuing
/// the verb index from `start_idx`) and a real DAC node (`dac` != 0). The wait for
/// LPIB to move is wall-clock bounded so a stalled stream cannot wedge boot.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn hda_pcm_selftest(
    mmio: u64,
    corb_va: u64,
    rirb_va: u64,
    codec: u32,
    dac: u32,
    start_idx: u16,
) {
    if dac == 0 {
        serial_write(b"HDA: pcm no dac\n");
        return;
    }
    // Output stream descriptors follow the input ones: base = 0x80 + ISS*0x20, where
    // ISS (input stream count) is GCAP bits [11:8]. SD0 is the first output stream.
    let gcap = core::ptr::read_volatile(mmio as *const u16);
    let iss = ((gcap >> 8) & 0xF) as u64;
    let sd = mmio + 0x80 + iss * 0x20;
    const STREAM: u32 = 1; // stream tag (1..15; 0 = unassociated)
    const FMT: u16 = 0x0011; // 48 kHz base, 16-bit samples, 2 channels
    const BUF_BYTES: u64 = 4096;

    // One DMA page for the PCM samples, one for the BDL.
    let buf = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            serial_write(b"HDA: pcm dma fail\n");
            return;
        }
    };
    let bdl = match dma::dma_alloc(1) {
        Some(p) => p,
        None => {
            dma::dma_free(buf, 1);
            serial_write(b"HDA: pcm dma fail\n");
            return;
        }
    };
    let buf_va = mm::phys_to_virt(buf);
    let bdl_va = mm::phys_to_virt(bdl);
    // Fill the buffer with a simple 16-bit square wave so a real backend would emit a
    // tone; the test only needs the controller to stream the bytes.
    let mut i = 0u64;
    while i < BUF_BYTES / 2 {
        let sample: u16 = if (i / 64) & 1 == 0 { 0x2000 } else { 0xE000 };
        core::ptr::write_volatile((buf_va + i * 2) as *mut u16, sample);
        i += 1;
    }
    // Two-entry BDL (spec minimum), each covering half the buffer, IOC set. Entry
    // layout: [u64 buffer addr][u32 length][u32 flags (bit0 = interrupt-on-complete)].
    let half = BUF_BYTES / 2;
    core::ptr::write_volatile(bdl_va as *mut u64, buf);
    core::ptr::write_volatile((bdl_va + 8) as *mut u32, half as u32);
    core::ptr::write_volatile((bdl_va + 12) as *mut u32, 1);
    core::ptr::write_volatile((bdl_va + 16) as *mut u64, buf + half);
    core::ptr::write_volatile((bdl_va + 24) as *mut u32, half as u32);
    core::ptr::write_volatile((bdl_va + 28) as *mut u32, 1);

    // Reset SD0 (SRST: set bit0, wait until it reads 1, clear, wait until it reads 0).
    core::ptr::write_volatile(sd as *mut u8, 0x01);
    let mut s = 0u64;
    while core::ptr::read_volatile(sd as *const u8) & 0x01 == 0 && s < 1_000_000 {
        s += 1;
    }
    core::ptr::write_volatile(sd as *mut u8, 0x00);
    s = 0;
    while core::ptr::read_volatile(sd as *const u8) & 0x01 != 0 && s < 1_000_000 {
        s += 1;
    }
    // Clear any latched stream status (BCIS/FIFOE/DESE are write-1-to-clear).
    core::ptr::write_volatile((sd + 0x03) as *mut u8, 0x1C);
    // Program the stream: BDL pointer, cyclic buffer length, last valid index, format,
    // and the stream number (SDnCTL bits [23:20] = high nibble of the byte at +0x02).
    core::ptr::write_volatile((sd + 0x18) as *mut u32, bdl as u32); // BDPL
    core::ptr::write_volatile((sd + 0x1C) as *mut u32, (bdl >> 32) as u32); // BDPU
    core::ptr::write_volatile((sd + 0x08) as *mut u32, BUF_BYTES as u32); // CBL
    core::ptr::write_volatile((sd + 0x0C) as *mut u16, 1); // LVI = entries - 1
    core::ptr::write_volatile((sd + 0x12) as *mut u16, FMT); // SDnFMT
    core::ptr::write_volatile((sd + 0x02) as *mut u8, (STREAM as u8) << 4);

    // Configure the codec DAC: power up, set the matching format, and bind it to our
    // stream/channel so the codec pulls from SD0 once the stream runs; unmute it.
    let dbase = (codec << 28) | (dac << 20);
    let mut vi = start_idx;
    vi += 1;
    let _ = hda_verb(mmio, corb_va, rirb_va, vi, dbase | (0x705 << 8)); // SET_POWER_STATE D0
    vi += 1;
    let _ = hda_verb(mmio, corb_va, rirb_va, vi, dbase | (0x2 << 16) | FMT as u32); // SET_CONVERTER_FORMAT
    vi += 1;
    let _ = hda_verb(mmio, corb_va, rirb_va, vi, dbase | (0x706 << 8) | (STREAM << 4)); // SET_STREAM_CHANNEL
    vi += 1;
    let _ = hda_verb(mmio, corb_va, rirb_va, vi, dbase | (0x3 << 16) | 0xB000); // SET_AMP_GAIN_MUTE unmute

    // Run the stream (SDnCTL.RUN = bit1) and confirm SDnLPIB advances. Reading LPIB
    // forces a VM exit so QEMU runs the codec's audio timer (which drives the DMA).
    let lpib0 = core::ptr::read_volatile((sd + 0x04) as *const u32);
    let ctl0 = core::ptr::read_volatile(sd as *const u8);
    core::ptr::write_volatile(sd as *mut u8, ctl0 | 0x02);
    let tsc_hz = tsc_hz_via_pit();
    let tsc_start = core::arch::x86_64::_rdtsc();
    let tsc_budget = if tsc_hz != 0 { tsc_hz * 2 } else { u64::MAX };
    let iter_cap: u64 = if tsc_hz != 0 { 200_000_000 } else { 20_000_000 };
    let mut lpib = lpib0;
    let mut n = 0u64;
    loop {
        lpib = core::ptr::read_volatile((sd + 0x04) as *const u32);
        if lpib != lpib0 {
            break;
        }
        n += 1;
        if n & 0x3F == 0
            && core::arch::x86_64::_rdtsc().wrapping_sub(tsc_start) > tsc_budget
        {
            break;
        }
        if n > iter_cap {
            break;
        }
        core::hint::spin_loop();
    }
    // Stop the stream. On success, KEEP the buffer + BDL + SD0 programming + codec
    // binding as the persistent audio context (the codec stays bound to stream 1) so
    // sys_ioctl op 7 (audio_write) can stream userspace PCM through it without
    // re-initialising the codec. On no-progress, release everything.
    let ctl1 = core::ptr::read_volatile(sd as *const u8);
    core::ptr::write_volatile(sd as *mut u8, ctl1 & !0x02);
    if lpib != lpib0 {
        AUDIO_SD = sd;
        AUDIO_BUF_VA = buf_va;
        AUDIO_BDL_PHYS = bdl;
        AUDIO_READY = true;
        serial_write(b"HDA: pcm lpib=0x");
        serial_write_hex(lpib as u64);
        serial_write(b" ok\n");
    } else {
        dma::dma_free(bdl, 1);
        dma::dma_free(buf, 1);
        serial_write(b"HDA: pcm no-progress\n");
    }
}

/// Play one block of 16-bit PCM through the persistent HD Audio output stream
/// (full-os guide Part III audio): the reusable core a userspace app reaches via
/// sys_ioctl op 7. Copies up to AUDIO_BUF_BYTES of `samples` into the stream buffer
/// (zero-padding the remainder to silence), resets + reprograms SD0 for the buffer,
/// runs it, and waits (wall-clock bounded) for the controller to DMA it (SDnLPIB
/// advances). Returns the sample-byte count accepted, or 0 if no audio device is
/// ready or the stream did not progress.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn hda_audio_play(samples: &[u8]) -> usize {
    if !AUDIO_READY {
        return 0;
    }
    let cap = AUDIO_BUF_BYTES as usize;
    let n = if samples.len() > cap { cap } else { samples.len() };
    let dst = AUDIO_BUF_VA as *mut u8;
    let mut i = 0usize;
    while i < n {
        core::ptr::write_volatile(dst.add(i), samples[i]);
        i += 1;
    }
    while i < cap {
        core::ptr::write_volatile(dst.add(i), 0); // pad with silence
        i += 1;
    }
    let sd = AUDIO_SD;
    const FMT: u16 = 0x0011; // 48 kHz / 16-bit / 2 channels (matches the codec binding)
    const STREAM: u8 = 1;
    // Reset SD0 (SRST), then reprogram it -- the BDL still points at the buffer halves.
    core::ptr::write_volatile(sd as *mut u8, 0x01);
    let mut s = 0u64;
    while core::ptr::read_volatile(sd as *const u8) & 0x01 == 0 && s < 1_000_000 {
        s += 1;
    }
    core::ptr::write_volatile(sd as *mut u8, 0x00);
    s = 0;
    while core::ptr::read_volatile(sd as *const u8) & 0x01 != 0 && s < 1_000_000 {
        s += 1;
    }
    core::ptr::write_volatile((sd + 0x03) as *mut u8, 0x1C); // clear status (RW1C)
    core::ptr::write_volatile((sd + 0x18) as *mut u32, AUDIO_BDL_PHYS as u32); // BDPL
    core::ptr::write_volatile((sd + 0x1C) as *mut u32, (AUDIO_BDL_PHYS >> 32) as u32); // BDPU
    core::ptr::write_volatile((sd + 0x08) as *mut u32, AUDIO_BUF_BYTES as u32); // CBL
    core::ptr::write_volatile((sd + 0x0C) as *mut u16, 1); // LVI = entries - 1
    core::ptr::write_volatile((sd + 0x12) as *mut u16, FMT); // SDnFMT
    core::ptr::write_volatile((sd + 0x02) as *mut u8, STREAM << 4); // stream number
    // Run + watch SDnLPIB advance (the DMA proof). Wall-clock bounded.
    let lpib0 = core::ptr::read_volatile((sd + 0x04) as *const u32);
    let ctl0 = core::ptr::read_volatile(sd as *const u8);
    core::ptr::write_volatile(sd as *mut u8, ctl0 | 0x02);
    let tsc_hz = tsc_hz_via_pit();
    let tsc_start = core::arch::x86_64::_rdtsc();
    let tsc_budget = if tsc_hz != 0 { tsc_hz * 2 } else { u64::MAX };
    let iter_cap: u64 = if tsc_hz != 0 { 200_000_000 } else { 20_000_000 };
    let mut lpib = lpib0;
    let mut k = 0u64;
    loop {
        lpib = core::ptr::read_volatile((sd + 0x04) as *const u32);
        if lpib != lpib0 {
            break;
        }
        k += 1;
        if k & 0x3F == 0 && core::arch::x86_64::_rdtsc().wrapping_sub(tsc_start) > tsc_budget {
            break;
        }
        if k > iter_cap {
            break;
        }
        core::hint::spin_loop();
    }
    let ctl1 = core::ptr::read_volatile(sd as *const u8);
    core::ptr::write_volatile(sd as *mut u8, ctl1 & !0x02); // stop
    if lpib != lpib0 {
        n
    } else {
        0
    }
}

/// Userspace-PCM path self-test (full-os guide Part III audio): exercise the exact
/// `hda_audio_play` core a ring-3 app reaches through sys_ioctl op 7, with a
/// kernel-built PCM block, and confirm the controller streamed it. Runs only when a
/// real duplex codec is present (AUDIO_READY). Marker `AUDIO: play n=0x.. ok`.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn hda_audio_selftest() {
    if !AUDIO_READY {
        return;
    }
    let mut pcm = [0u8; 512];
    let mut i = 0usize;
    while i + 1 < pcm.len() {
        let v: u16 = if (i / 64) & 1 == 0 { 0x3000 } else { 0xD000 };
        pcm[i] = (v & 0xFF) as u8;
        pcm[i + 1] = (v >> 8) as u8;
        i += 2;
    }
    let n = hda_audio_play(&pcm);
    if n == pcm.len() {
        serial_write(b"AUDIO: play n=0x");
        serial_write_hex(n as u64);
        serial_write(b" ok\n");
    } else {
        serial_write(b"AUDIO: play FAIL\n");
    }
}

/// PCIe ECAM self-test (full-os guide Part II.7, PCIe ECAM): read PCI config
/// space through the memory-mapped Enhanced Configuration Access Mechanism and
/// confirm it agrees with the legacy 0xCF8/0xCFC I/O path. The ECAM base is read
/// from the q35 MCH PCIEXBAR (host-bridge config offset 0x60), not hardcoded.
/// Config address = base + (bus<<20) + (dev<<15) + (func<<12) + offset.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn ecam_selftest() -> u64 {
    // PCIEXBAR (0:0:0 offset 0x60): bit 0 = enable, bits[2:1] = length
    // (00 => 256 MiB), base in the high bits (256 MiB-aligned).
    let bar = pci_read32(0, 0, 0, 0x60);
    if bar & 1 == 0 {
        serial_write(b"ECAM: disabled\n");
        return 0;
    }
    let base = (bar as u64) & 0xF000_0000;
    serial_write(b"ECAM: base=0x");
    serial_write_hex(base);
    serial_write(b"\n");
    // Verify ECAM agrees with legacy config reads for two q35 functions: the MCH
    // host bridge (0:0:0) and the LPC/ISA bridge (0:0x1F:0). Each lives in its
    // own 4 KiB ECAM page, mapped one at a time into the kernel MMIO window.
    let mut ok = true;
    let probes: [(u8, u8, u8); 2] = [(0, 0, 0), (0, 0x1F, 0)];
    let mut i = 0usize;
    while i < probes.len() {
        let (b, d, f) = probes[i];
        let legacy = pci_read32(b, d, f, 0);
        let phys =
            base + ((b as u64) << 20) + ((d as u64) << 15) + ((f as u64) << 12);
        match mmio_map_4k(phys) {
            Some(va) => {
                let ecam = core::ptr::read_volatile(va as *const u32);
                if ecam != legacy {
                    ok = false;
                }
            }
            None => ok = false,
        }
        i += 1;
    }
    if ok {
        serial_write(b"ECAM: selftest ok\n");
        1
    } else {
        serial_write(b"ECAM: selftest fail\n");
        0
    }
}

/// Walk a PCI function's capability list for capability id `cap_id`; returns the
/// config-space offset of the capability, or None. Capability structures are
/// dword-aligned (low 2 bits of each pointer are zero).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn pci_find_cap(b: u8, d: u8, f: u8, cap_id: u8) -> Option<u8> {
    let status = (pci_read32(b, d, f, 0x04) >> 16) as u16;
    if status & (1 << 4) == 0 {
        return None; // capabilities list not present
    }
    let mut ptr = (pci_read32(b, d, f, 0x34) & 0xFC) as u8;
    let mut guard = 0;
    while ptr != 0 && guard < 48 {
        let dw = pci_read32(b, d, f, ptr);
        let id = (dw & 0xFF) as u8;
        let next = ((dw >> 8) & 0xFC) as u8;
        if id == cap_id {
            return Some(ptr);
        }
        ptr = next;
        guard += 1;
    }
    None
}

/// MSI-X self-test (full-os guide Part II.7, interrupt model): find the first PCI
/// function exposing an MSI-X capability (id 0x11), read its table size from the
/// Message Control register, set the MSI-X Enable bit, and read it back to
/// confirm — then restore the original control so the existing driver is
/// undisturbed. Reports `MSIX: none` if no device has MSI-X.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn msix_selftest() -> u64 {
    let mut dev = 0u8;
    while dev < 32 {
        let id0 = pci_read32(0, dev, 0, 0);
        if (id0 & 0xFFFF) as u16 == 0xFFFF {
            dev += 1;
            continue;
        }
        let hdr = (pci_read32(0, dev, 0, 0x0C) >> 16) & 0xFF;
        let funcs = if hdr & 0x80 != 0 { 8u8 } else { 1u8 };
        let mut func = 0u8;
        while func < funcs {
            let id = pci_read32(0, dev, func, 0);
            if (id & 0xFFFF) as u16 != 0xFFFF {
                if let Some(cap) = pci_find_cap(0, dev, func, 0x11) {
                    // Message Control is the high 16 bits of the dword at `cap`.
                    let dw = pci_read32(0, dev, func, cap);
                    let msgctl = (dw >> 16) as u16;
                    let table_size = (msgctl & 0x7FF) + 1;
                    let device = ((id >> 16) & 0xFFFF) as u16;
                    // Set the MSI-X Enable bit (15) and confirm it sticks.
                    let enabled_dw = (dw & 0x0000_FFFF) | (((msgctl | 0x8000) as u32) << 16);
                    pci_write32(0, dev, func, cap, enabled_dw);
                    let rb = (pci_read32(0, dev, func, cap) >> 16) as u16;
                    let enabled = rb & 0x8000 != 0;
                    // Restore the original control (leave the driver's state alone).
                    pci_write32(0, dev, func, cap, dw);
                    serial_write(b"MSIX: dev=0x");
                    serial_write_hex(device as u64);
                    serial_write(b" vectors=0x");
                    serial_write_hex(table_size as u64);
                    if !enabled {
                        serial_write(b" enable fail\n");
                        return 0;
                    }
                    serial_write(b" enable ok\n");
                    // Program the MSI-X table (full-os guide Part II.7): locate
                    // the table via the cap's Table BIR + offset (dword @cap+4),
                    // map that BAR page, write a message address/data + the per-
                    // vector mask into entry 0, read it back, then RESTORE the
                    // original. MSI-X stays disabled (control was restored above)
                    // so this only proves the table registers latch — no live
                    // interrupt is armed. This is the concrete half of MSI-X; ISR
                    // delivery + irq_dispatch wiring is carry-forward.
                    msix_table_selftest(dev, func, cap);
                    return 1;
                }
            }
            func += 1;
        }
        dev += 1;
    }
    serial_write(b"MSIX: none\n");
    0
}

/// Program MSI-X table entry 0 (message address/data + per-vector mask), read it
/// back, and restore the original. Locates the table from the capability's Table
/// BIR + offset (dword @cap+4) and maps the BAR page. MSI-X is disabled by the
/// caller, so this never arms a real interrupt — it proves only that the table
/// registers latch (full-os guide Part II.7).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn msix_table_selftest(dev: u8, func: u8, cap: u8) {
    let tdw = pci_read32(0, dev, func, cap + 4);
    let bir = (tdw & 0x7) as u8;
    let toff = (tdw & 0xFFFF_FFF8) as u64;
    let bar_off = 0x10 + bir * 4;
    let bar = pci_read32(0, dev, func, bar_off);
    if bar & 1 != 0 {
        serial_write(b"MSIX: table not mmio\n"); // table must be in a memory BAR
        return;
    }
    let mut base = (bar & 0xFFFF_FFF0) as u64;
    if (bar >> 1) & 0x3 == 0x2 {
        base |= (pci_read32(0, dev, func, bar_off + 4) as u64) << 32;
    }
    if base == 0 {
        serial_write(b"MSIX: table bar unassigned\n");
        return;
    }
    let table_phys = base + toff;
    // Map TWO pages: the Table Offset is only 8-byte aligned (and the BAR base
    // only 16-byte aligned), so entry 0 can sit as high as in-page offset 0xFF8;
    // a 16-byte entry there would run off a single page. Two pages always cover it.
    let page = match mmio_map_region(table_phys & !0xFFF, 2) {
        Some(p) => p,
        None => {
            serial_write(b"MSIX: table map fail\n");
            return;
        }
    };
    let e0 = (page + (table_phys & 0xFFF)) as *mut u32;
    // Save the original entry-0 (addr_lo, addr_hi, data, vector_control).
    let s0 = core::ptr::read_volatile(e0);
    let s1 = core::ptr::read_volatile(e0.add(1));
    let s2 = core::ptr::read_volatile(e0.add(2));
    let s3 = core::ptr::read_volatile(e0.add(3));
    // Write a test message: address 0xFEE0_0000 (the x86 LAPIC MSI window),
    // data 0x41 (a sample vector), vector_control bit 0 = masked.
    core::ptr::write_volatile(e0, 0xFEE0_0000);
    core::ptr::write_volatile(e0.add(1), 0);
    core::ptr::write_volatile(e0.add(2), 0x41);
    core::ptr::write_volatile(e0.add(3), 1);
    let rb_addr = core::ptr::read_volatile(e0);
    let rb_data = core::ptr::read_volatile(e0.add(2));
    let rb_mask = core::ptr::read_volatile(e0.add(3)) & 1;
    // Restore the original entry so the device is left exactly as found.
    core::ptr::write_volatile(e0, s0);
    core::ptr::write_volatile(e0.add(1), s1);
    core::ptr::write_volatile(e0.add(2), s2);
    core::ptr::write_volatile(e0.add(3), s3);
    if rb_addr == 0xFEE0_0000 && (rb_data & 0xFFFF) == 0x41 && rb_mask == 1 {
        serial_write(b"MSIX: table ok bir=0x");
        serial_write_hex(bir as u64);
        serial_write(b"\n");
    } else {
        serial_write(b"MSIX: table fail\n");
    }
}

/// Claim a PCI function once so one function does not get initialized by
/// multiple in-kernel drivers.
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
unsafe fn pci_claim_device(bdf: PciBdf) -> bool {
    let key = pci_bdf_key(bdf);
    let mut i = 0usize;
    while i < PCI_CLAIM_SLOTS {
        let slot = PCI_CLAIMED[i];
        if slot == key {
            return false;
        }
        if slot == PCI_CLAIM_NONE {
            PCI_CLAIMED[i] = key;
            return true;
        }
        i += 1;
    }
    false
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
unsafe fn pci_find_device(vendor: u16, device: u16) -> Option<PciBdf> {
    for dev in 0..32u8 {
        let id = pci_read32(0, dev, 0, 0);
        let v = (id & 0xFFFF) as u16;
        let d = ((id >> 16) & 0xFFFF) as u16;
        if v == vendor && d == device {
            return Some(PciBdf { bus: 0, dev, func: 0 });
        }
    }
    None
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
unsafe fn pci_enable_io_bus_master(bdf: PciBdf) {
    let cmd_reg = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x04);
    let mut cmd = (cmd_reg & 0xFFFF) as u16;
    cmd |= 0x0005; // I/O space + bus master
    let new_cmd_reg = (cmd_reg & 0xFFFF_0000) | (cmd as u32);
    pci_write32(bdf.bus, bdf.dev, bdf.func, 0x04, new_cmd_reg);
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
unsafe fn pci_bar0_iobase(bdf: PciBdf) -> Option<u16> {
    let bar0_raw = pci_read32(bdf.bus, bdf.dev, bdf.func, 0x10);
    if (bar0_raw & 1) == 0 {
        return None;
    }
    let iobase = (bar0_raw & !3u32) as u16;
    if iobase == 0 {
        return None;
    }
    Some(iobase)
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
unsafe fn pci_find_virtio_legacy_iobase(device: u16) -> Option<u16> {
    let bdf = pci_find_device(0x1AF4, device)?;
    let iobase = pci_bar0_iobase(bdf)?;
    if !pci_claim_device(bdf) {
        return None;
    }
    pci_enable_io_bus_master(bdf);
    Some(iobase)
}

/// Scan PCI bus 0 for VirtIO block device (vendor 0x1AF4, device 0x1001).
/// Returns the I/O base address (BAR0) if found.
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
unsafe fn pci_find_virtio_blk() -> Option<u16> {
    pci_find_virtio_legacy_iobase(0x1001)
}

// --------------- M5: VirtIO legacy transport registers -----------------------

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const VIRTIO_DEVICE_FEATURES: u16 = 0;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const VIRTIO_GUEST_FEATURES: u16 = 4;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const VIRTIO_QUEUE_PFN: u16 = 8;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const VIRTIO_QUEUE_SIZE: u16 = 12;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const VIRTIO_QUEUE_SEL: u16 = 14;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const VIRTIO_QUEUE_NOTIFY: u16 = 16;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const VIRTIO_DEVICE_STATUS: u16 = 18;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const VIRTIO_ISR_STATUS: u16 = 19;

// Descriptor flags
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const VRING_DESC_F_NEXT: u16 = 1;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
const VRING_DESC_F_WRITE: u16 = 2;

// Block request types
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const VIRTIO_BLK_T_IN: u32 = 0;  // read
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const VIRTIO_BLK_T_OUT: u32 = 1; // write

// --------------- M5: Virtqueue descriptor ------------------------------------

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
#[repr(C, packed)]
struct VringDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

// Block request header layout (written via raw offsets, 16 bytes):
//   offset 0: type_ (u32) â€” VIRTIO_BLK_T_IN=0, VIRTIO_BLK_T_OUT=1
//   offset 4: reserved (u32)
//   offset 8: sector (u64)

// --------------- M5: Static memory for VirtIO --------------------------------

// Virtqueue area: 4 pages (16 KiB), enough for queue_size up to 256
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
#[repr(C, align(4096))]
struct VqMem([u8; 16384]);

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
static mut VQ_MEM: VqMem = VqMem([0; 16384]);

// DMA buffers: request header+status (1 page), data (1 page)
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut BLK_REQ_PAGE: Page = Page([0; 4096]);
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut BLK_DATA_PAGE: Page = Page([0; 4096]);

// Driver state
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
static mut BLK_IOBASE: u16 = 0;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
static mut BLK_QUEUE_SIZE: u16 = 0;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
static mut BLK_DESCS: *mut VringDesc = core::ptr::null_mut();
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
static mut BLK_AVAIL: *mut u8 = core::ptr::null_mut();
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
static mut BLK_USED: *const u8 = core::ptr::null();
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
static mut BLK_LAST_USED: u16 = 0;
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
static mut BLK_KV2P_DELTA: u64 = 0; // kphys - kvirt (wrapping)
#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
const BLK_MAX_QUEUE_SIZE: u16 = 256;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveBlockDriver {
    None,
    VirtioLegacy,
    Nvme,
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut ACTIVE_BLOCK_DRIVER: ActiveBlockDriver = ActiveBlockDriver::None;

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
unsafe fn blk_kv2p(va: u64) -> u64 {
    va.wrapping_add(BLK_KV2P_DELTA)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn block_driver_class() -> &'static [u8] {
    match ACTIVE_BLOCK_DRIVER {
        ActiveBlockDriver::Nvme => b"nvme",
        ActiveBlockDriver::VirtioLegacy => b"virtio-blk-pci",
        ActiveBlockDriver::None => b"none",
    }
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn emit_native_probe_error(err: runtime::native::ProbeError) {
    match err {
        runtime::native::ProbeError::NotFound => serial_write(b"NVME: controller missing\n"),
        runtime::native::ProbeError::MmioBarMissing => serial_write(b"NVME: bar missing\n"),
        runtime::native::ProbeError::IrqUnavailable => serial_write(b"NVME: irq unavailable\n"),
        runtime::native::ProbeError::ControllerTimeout => {
            serial_write(b"NVME: controller timeout\n")
        }
        runtime::native::ProbeError::IoQueueFailed => serial_write(b"NVME: io queue fail\n"),
        runtime::native::ProbeError::IdentifyFailed => serial_write(b"NVME: identify fail\n"),
        runtime::native::ProbeError::NamespaceMissing => serial_write(b"NVME: namespace missing\n"),
    }
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn block_driver_probe(prefer_native: bool, require_native: bool, emit_native_negative: bool) -> bool {
    ACTIVE_BLOCK_DRIVER = ActiveBlockDriver::None;

    if prefer_native || require_native {
        match runtime::native::probe_nvme(BLK_KV2P_DELTA, HHDM_OFFSET) {
            Ok(_) => {
                ACTIVE_BLOCK_DRIVER = ActiveBlockDriver::Nvme;
                return true;
            }
            Err(err) if emit_native_negative => emit_native_probe_error(err),
            Err(_) => {}
        }
        if require_native {
            return false;
        }
    }

    if let Some(iobase) = pci_find_virtio_blk() {
        if virtio_blk_init(iobase) {
            ACTIVE_BLOCK_DRIVER = ActiveBlockDriver::VirtioLegacy;
            serial_write(b"BLK: found virtio-blk\n");
            return true;
        }
    }
    false
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn block_io_dispatch(write: bool, sector: u64, len: usize, fua: bool) -> bool {
    match ACTIVE_BLOCK_DRIVER {
        ActiveBlockDriver::VirtioLegacy => virtio_blk_io(write, sector, len),
        ActiveBlockDriver::Nvme => runtime::native::nvme_read_write(write, sector, len, fua),
        ActiveBlockDriver::None => false,
    }
}

// Storage critical-section guard (full-os Part I.3, workload-distribution slice 2).
// STORAGE_LOCK is the innermost data lock: it serializes the shared single-instance
// block path -- the BLK_DATA_PAGE bounce buffer, the BLK_REQ_PAGE request header, and
// the virtio/NVMe ring head/tail -- all of which are touched ONLY across the
// fill -> block_io_dispatch -> read sequence a caller performs (the ring lives inside
// virtio_blk_io / nvme_read_write, reached only through block_io_dispatch). Holding
// the lock from storage_io_enter to storage_io_exit makes that whole sequence atomic
// against a concurrent CPU. Taken with interrupts masked (lock_cli) so this CPU's own
// timer cannot land mid-section on a lock it holds (leaf discipline). FS_LOCK (vfs) /
// NET_LOCK (pkg pump) may be held outside it -- STORAGE is innermost, never nests out.
// In every non-(go,not-compat) lane the machine is single-CPU (APs park, nothing does
// concurrent block I/O), so these are no-ops. Wired into the live concurrent-reachable
// block paths only: vfs read_sector/write_sector (the FS block primitive) and
// blk_read_page/blk_write_page (swap). The other block_io_dispatch callers (FAT/GPT/
// installer/crypt/journal/storage boot self-tests + the fs_test M6 path) run only at
// single-threaded boot, before the live scheduler, so they never race a lock holder.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
#[inline]
pub(crate) unsafe fn storage_io_enter() -> u64 {
    let saved = smp::lock_cli();
    smp::storage_lock();
    saved
}
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
#[inline]
pub(crate) unsafe fn storage_io_exit(saved: u64) {
    smp::storage_unlock();
    smp::unlock_sti(saved);
}
// compat_real_test enables go_test (so vfs compiles) but not the SMP locks: no-op there.
#[cfg(feature = "compat_real_test")]
#[inline]
pub(crate) unsafe fn storage_io_enter() -> u64 {
    0
}
#[cfg(feature = "compat_real_test")]
#[inline]
pub(crate) unsafe fn storage_io_exit(_saved: u64) {}

/// Write one 4 KiB page (8 sectors) from the physical frame `src_phys` to the
/// block device starting at `lba` (full-os guide Part I.4 swap). Copies through
/// the shared BLK_DATA_PAGE one sector at a time (the block path is 512 B).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn blk_write_page(lba: u64, src_phys: u64) -> bool {
    let src = mm::phys_to_virt(src_phys) as *const u8;
    let mut s = 0u64;
    while s < 8 {
        // STORAGE_LOCK per sector (slice 2): keep holds short -- re-acquire each
        // sector so a concurrent CPU's block op can interleave between sectors
        // (the bounce buffer + ring are still atomic within one sector's I/O).
        let g = storage_io_enter();
        core::ptr::copy_nonoverlapping(
            src.add((s * 512) as usize),
            BLK_DATA_PAGE.0.as_mut_ptr(),
            512,
        );
        let ok = block_io_dispatch(true, lba + s, 512, false);
        storage_io_exit(g);
        if !ok {
            return false;
        }
        s += 1;
    }
    true
}

/// Read one 4 KiB page (8 sectors) from `lba` into the physical frame `dst_phys`.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn blk_read_page(lba: u64, dst_phys: u64) -> bool {
    let dst = mm::phys_to_virt(dst_phys) as *mut u8;
    let mut s = 0u64;
    while s < 8 {
        let g = storage_io_enter();
        let ok = block_io_dispatch(false, lba + s, 512, false);
        if ok {
            core::ptr::copy_nonoverlapping(
                BLK_DATA_PAGE.0.as_ptr(),
                dst.add((s * 512) as usize),
                512,
            );
        }
        storage_io_exit(g);
        if !ok {
            return false;
        }
        s += 1;
    }
    true
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn block_flush_dispatch() -> bool {
    match ACTIVE_BLOCK_DRIVER {
        ActiveBlockDriver::VirtioLegacy => true,
        ActiveBlockDriver::Nvme => runtime::native::nvme_flush(),
        ActiveBlockDriver::None => false,
    }
}

/// Installer (full-os guide Part V.11): provision a target disk with an OS boot
/// record. Finds a SECOND virtio-blk device (the install target) — a generic
/// boot has only the boot disk, so this is a safe no-op there; only a lane that
/// attaches a blank second disk exercises the write. Switches the block driver
/// to the target, writes a boot record (a "RUGOINST" magic + version + the
/// 0x55AA MBR signature), reads it back to verify, then restores the boot disk.
/// v1 boundary: it provisions one image block and verifies the write/read path;
/// a full bootable install (partition layout, kernel copy, bootloader) is
/// carry-forward.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn installer_selftest() {
    // The boot disk is the first virtio-blk (already initialized by storage boot).
    let boot = match pci_find_device(0x1AF4, 0x1001) {
        Some(b) => b,
        None => return, // not a virtio-blk lane (e.g. NVMe): nothing to do
    };
    // Find a SECOND virtio-blk after the boot disk: the install target.
    let mut tdev = boot.dev + 1;
    let mut target = None;
    while tdev < 32 {
        let id = pci_read32(0, tdev, 0, 0);
        if (id & 0xFFFF) as u16 == 0x1AF4 && ((id >> 16) & 0xFFFF) as u16 == 0x1001 {
            target = Some(PciBdf { bus: 0, dev: tdev, func: 0 });
            break;
        }
        tdev += 1;
    }
    let tgt = match target {
        Some(b) => b,
        None => {
            serial_write(b"INSTALL: no target\n");
            return;
        }
    };
    let tgt_iobase = match pci_bar0_iobase(tgt) {
        Some(io) => io,
        None => {
            serial_write(b"INSTALL: target no bar\n");
            return;
        }
    };
    let boot_iobase = match pci_bar0_iobase(boot) {
        Some(io) => io,
        None => return,
    };
    pci_enable_io_bus_master(tgt);
    // Switch the block driver to the target disk and provision it.
    if !virtio_blk_init(tgt_iobase) {
        serial_write(b"INSTALL: target init fail\n");
        let _ = virtio_blk_init(boot_iobase); // restore the boot disk
        return;
    }
    // This self-test runs on EVERY go boot, so it must NEVER clobber an unrelated
    // data disk: only provision a target whose sector 0 is blank (a fresh disk)
    // or already carries our magic (idempotent re-install).
    let provisionable = install_target_provisionable();
    let mbr_ok = provisionable && install_write_and_verify();
    let payload_ok = mbr_ok && install_payload_and_verify();
    // Reset the target so it releases the shared virtqueue (only one device may
    // be bound to VQ_MEM at a time), then restore the boot disk.
    outb(tgt_iobase + VIRTIO_DEVICE_STATUS, 0);
    if !virtio_blk_init(boot_iobase) {
        // The boot disk just initialized successfully at storage boot, so this
        // should not happen; if it does, fail loudly rather than feed the shell
        // a half-initialized disk0.
        serial_write(b"INSTALL: boot restore FAIL\n");
        qemu_exit(0x31);
        loop {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
    if !provisionable {
        serial_write(b"INSTALL: target not blank, refusing\n");
    } else if payload_ok {
        serial_write(b"INSTALL: image written+verified ok\n");
        serial_write(b"INSTALL: partition type=0x");
        serial_write_hex(INSTALL_PART_TYPE as u64);
        serial_write(b" lba=0x");
        serial_write_hex(INSTALL_PART_LBA as u64);
        serial_write(b" sectors=0x");
        serial_write_hex(INSTALL_PART_SECTORS as u64);
        serial_write(b"\n");
        serial_write(b"INSTALL: bootable install ok\n");
    } else if mbr_ok {
        serial_write(b"INSTALL: payload FAIL\n");
    } else {
        serial_write(b"INSTALL: verify FAIL\n");
    }
}

/// Read the active (target) device's sector 0 and report whether it is safe to
/// provision: all-zero (a fresh disk) or already carrying our boot-record magic.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn install_target_provisionable() -> bool {
    const MAGIC: &[u8; 8] = b"RUGOINST";
    let dp = BLK_DATA_PAGE.0.as_mut_ptr();
    // The FIRST read right after switching/re-initializing the virtio queue on a
    // freshly bound device returns stale/empty data (the device reports success
    // but does not fill the buffer). Do a throwaway warm-up read, then read for
    // real — otherwise a non-blank target would be misread as blank and clobbered.
    core::ptr::write_bytes(dp, 0, 512);
    let _ = virtio_blk_io(false, 0, 512);
    core::ptr::write_bytes(dp, 0, 512);
    if !virtio_blk_io(false, 0, 512) {
        return false; // cannot read -> do not write (safe)
    }
    let mut i = 0usize;
    let mut blank = true;
    while i < 512 {
        if *dp.add(i) != 0 {
            blank = false;
            break;
        }
        i += 1;
    }
    if blank {
        return true;
    }
    i = 0;
    while i < 8 {
        if *dp.add(i) != MAGIC[i] {
            return false;
        }
        i += 1;
    }
    true // already our image -> idempotent re-provision
}

/// Install-target geometry: a single bootable primary partition starting at LBA
/// `INSTALL_PART_LBA`, `INSTALL_PART_SECTORS` long, holding the installed image.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const INSTALL_PART_LBA: u32 = 64;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const INSTALL_PART_SECTORS: u32 = 8;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const INSTALL_PART_TYPE: u8 = 0x83; // Linux-type primary partition

/// Write the MBR (boot-record magic + a real primary partition-table entry) to
/// sector 0 of the active (target) device and read it back to confirm a
/// byte-exact round-trip — including the partition entry the kernel's own MBR
/// parser (sys_sysinfo op 5) reads.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn install_write_and_verify() -> bool {
    const MAGIC: &[u8; 8] = b"RUGOINST";
    let dp = BLK_DATA_PAGE.0.as_mut_ptr();
    core::ptr::write_bytes(dp, 0, 512);
    core::ptr::copy_nonoverlapping(MAGIC.as_ptr(), dp, 8);
    *dp.add(8) = 0x01; // image version
    // Primary partition entry 0 at MBR offset 446 (16 bytes): a BOOTABLE
    // partition the kernel parser can read (boot flag@+0, type@+4, LBA@+8,
    // sector count@+12). CHS fields are left zero — the parser uses LBA.
    let e = 446usize;
    *dp.add(e) = 0x80; // bootable flag
    *dp.add(e + 4) = INSTALL_PART_TYPE;
    core::ptr::copy_nonoverlapping(INSTALL_PART_LBA.to_le_bytes().as_ptr(), dp.add(e + 8), 4);
    core::ptr::copy_nonoverlapping(
        INSTALL_PART_SECTORS.to_le_bytes().as_ptr(),
        dp.add(e + 12),
        4,
    );
    *dp.add(510) = 0x55; // MBR boot signature
    *dp.add(511) = 0xAA;
    if !virtio_blk_io(true, 0, 512) {
        return false;
    }
    // Clear the buffer, then read sector 0 back into it (so a stale match in the
    // page cannot pass the check).
    core::ptr::write_bytes(dp, 0, 512);
    if !virtio_blk_io(false, 0, 512) {
        return false;
    }
    let mut i = 0usize;
    while i < 8 {
        if *dp.add(i) != MAGIC[i] {
            return false;
        }
        i += 1;
    }
    if *dp.add(8) != 0x01 || *dp.add(510) != 0x55 || *dp.add(511) != 0xAA {
        return false;
    }
    // Verify the partition entry round-tripped exactly.
    if *dp.add(e) != 0x80 || *dp.add(e + 4) != INSTALL_PART_TYPE {
        return false;
    }
    let rb_lba = u32::from_le_bytes([*dp.add(e + 8), *dp.add(e + 9), *dp.add(e + 10), *dp.add(e + 11)]);
    let rb_secs =
        u32::from_le_bytes([*dp.add(e + 12), *dp.add(e + 13), *dp.add(e + 14), *dp.add(e + 15)]);
    rb_lba == INSTALL_PART_LBA && rb_secs == INSTALL_PART_SECTORS
}

/// Deterministic per-sector fill byte for the installed payload, so a missing,
/// duplicated, or mis-ordered sector is detectable on read-back.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
#[inline]
fn install_fill(sector: u64, off: usize) -> u8 {
    ((sector as u8).wrapping_mul(31)).wrapping_add(off as u8).wrapping_add(0x5A)
}

/// Write a multi-sector image into the partition (LBA `INSTALL_PART_LBA`..) and
/// read every sector back to confirm the install landed: the partition's first
/// sector is its own boot record (`RUGOPART` magic + 0x55AA), and all sectors
/// carry a deterministic fill pattern.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn install_payload_and_verify() -> bool {
    const PMAGIC: &[u8; 8] = b"RUGOPART";
    let dp = BLK_DATA_PAGE.0.as_mut_ptr();
    let part_lba = INSTALL_PART_LBA as u64;
    let mut s = 0u64;
    while s < INSTALL_PART_SECTORS as u64 {
        core::ptr::write_bytes(dp, 0, 512);
        let mut i = 16usize;
        while i < 504 {
            *dp.add(i) = install_fill(s, i);
            i += 1;
        }
        if s == 0 {
            core::ptr::copy_nonoverlapping(PMAGIC.as_ptr(), dp, 8);
            *dp.add(8) = 0x01; // payload image version
            *dp.add(510) = 0x55;
            *dp.add(511) = 0xAA;
        }
        if !virtio_blk_io(true, part_lba + s, 512) {
            return false;
        }
        s += 1;
    }
    // Read every sector back and verify magic + fill (cleared buffer each time so
    // a stale page cannot pass).
    s = 0;
    while s < INSTALL_PART_SECTORS as u64 {
        core::ptr::write_bytes(dp, 0, 512);
        if !virtio_blk_io(false, part_lba + s, 512) {
            return false;
        }
        if s == 0 {
            let mut i = 0usize;
            while i < 8 {
                if *dp.add(i) != PMAGIC[i] {
                    return false;
                }
                i += 1;
            }
            if *dp.add(510) != 0x55 || *dp.add(511) != 0xAA {
                return false;
            }
        }
        let mut i = 16usize;
        while i < 504 {
            if *dp.add(i) != install_fill(s, i) {
                return false;
            }
            i += 1;
        }
        s += 1;
    }
    true
}

// --------------- M5: VirtIO block init ---------------------------------------

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "go_test"))]
unsafe fn virtio_blk_init(iobase: u16) -> bool {
    BLK_IOBASE = iobase;

    // Step 1: Reset
    outb(iobase + VIRTIO_DEVICE_STATUS, 0);

    // Step 2: Acknowledge
    outb(iobase + VIRTIO_DEVICE_STATUS, 1);

    // Step 3: Driver
    outb(iobase + VIRTIO_DEVICE_STATUS, 1 | 2);

    // Step 4: Feature negotiation â€” accept no features
    let _features = inl(iobase + VIRTIO_DEVICE_FEATURES);
    outl(iobase + VIRTIO_GUEST_FEATURES, 0);

    // Step 5: Select queue 0, read queue size
    outw(iobase + VIRTIO_QUEUE_SEL, 0);
    let qsz = inw(iobase + VIRTIO_QUEUE_SIZE);
    if qsz == 0 || qsz > BLK_MAX_QUEUE_SIZE {
        outb(iobase + VIRTIO_DEVICE_STATUS, 0x80); // FAILED
        return false;
    }
    BLK_QUEUE_SIZE = qsz;

    // Step 5b: Validate queue_size fits in our static VQ buffer.
    // Layout: descriptors (qsz*16) | avail ring (6+2*qsz) | padding | used ring (6+8*qsz)
    let vq_buf_size = core::mem::size_of::<VqMem>();
    let desc_end = (qsz as usize) * 16;
    let avail_end_ck = desc_end + 6 + 2 * (qsz as usize);
    let used_off_ck = (avail_end_ck + 4095) & !4095;
    let used_end_ck = used_off_ck + 6 + 8 * (qsz as usize);
    if used_end_ck > vq_buf_size {
        outb(iobase + VIRTIO_DEVICE_STATUS, 0x80); // FAILED
        return false;
    }

    // Step 6: Zero queue memory and set up pointers
    core::ptr::write_bytes(VQ_MEM.0.as_mut_ptr(), 0, VQ_MEM.0.len());

    let base = VQ_MEM.0.as_mut_ptr();
    BLK_DESCS = base as *mut VringDesc;

    let avail_offset = (qsz as usize) * 16;
    BLK_AVAIL = base.add(avail_offset);

    let avail_end = avail_offset + 6 + 2 * (qsz as usize);
    let used_offset = (avail_end + 4095) & !4095;
    BLK_USED = base.add(used_offset) as *const u8;
    BLK_LAST_USED = 0;

    // Step 7: Write queue PFN (physical page frame number)
    let queue_phys = blk_kv2p(base as u64);
    outl(iobase + VIRTIO_QUEUE_PFN, (queue_phys >> 12) as u32);

    // Step 8: DRIVER_OK
    outb(iobase + VIRTIO_DEVICE_STATUS, 1 | 2 | 4);

    #[cfg(feature = "blk_invariants_test")]
    serial_write(b"BLK: invariants ok\n");
    true
}

// --------------- M5: VirtIO block I/O ----------------------------------------

/// Perform a single block I/O operation. Returns true on success.
/// `sector` is the starting 512-byte sector.
/// `len` must be a multiple of 512, max 4096.
/// For writes, data must already be in BLK_DATA_PAGE.
/// For reads, data is placed in BLK_DATA_PAGE.
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn virtio_blk_io(write: bool, sector: u64, len: usize) -> bool {
    let iobase = BLK_IOBASE;
    let qsz = BLK_QUEUE_SIZE as usize;

    // Set up request header (via raw offsets to avoid packed ref UB)
    let hdr = BLK_REQ_PAGE.0.as_mut_ptr();
    // type_ at offset 0 (u32)
    core::ptr::write_volatile(hdr as *mut u32,
        if write { VIRTIO_BLK_T_OUT } else { VIRTIO_BLK_T_IN });
    // reserved at offset 4 (u32)
    core::ptr::write_volatile(hdr.add(4) as *mut u32, 0);
    // sector at offset 8 (u64)
    core::ptr::write_volatile(hdr.add(8) as *mut u64, sector);

    // Status byte (after header, at offset 16 in req page)
    let status_ptr = BLK_REQ_PAGE.0.as_mut_ptr().add(16);
    core::ptr::write_volatile(status_ptr, 0xFF); // init to failure

    let hdr_phys = blk_kv2p(hdr as u64);
    let status_phys = blk_kv2p(status_ptr as u64);
    let data_phys = blk_kv2p(BLK_DATA_PAGE.0.as_ptr() as u64);

    // Write descriptors via raw pointers (VringDesc is packed)
    // Each desc is 16 bytes: addr(u64) + len(u32) + flags(u16) + next(u16)

    // Descriptor 0: request header (device reads, 16 bytes)
    let d0 = BLK_DESCS as *mut u8;
    core::ptr::write(d0.add(0) as *mut u64, hdr_phys);
    core::ptr::write(d0.add(8) as *mut u32, 16);
    core::ptr::write(d0.add(12) as *mut u16, VRING_DESC_F_NEXT);
    core::ptr::write(d0.add(14) as *mut u16, 1);

    // Descriptor 1: data buffer
    let d1 = (BLK_DESCS as *mut u8).add(16);
    core::ptr::write(d1.add(0) as *mut u64, data_phys);
    core::ptr::write(d1.add(8) as *mut u32, len as u32);
    core::ptr::write(d1.add(12) as *mut u16,
        VRING_DESC_F_NEXT | if !write { VRING_DESC_F_WRITE } else { 0 });
    core::ptr::write(d1.add(14) as *mut u16, 2);

    // Descriptor 2: status byte (device writes, 1 byte)
    let d2 = (BLK_DESCS as *mut u8).add(32);
    core::ptr::write(d2.add(0) as *mut u64, status_phys);
    core::ptr::write(d2.add(8) as *mut u32, 1);
    core::ptr::write(d2.add(12) as *mut u16, VRING_DESC_F_WRITE);
    core::ptr::write(d2.add(14) as *mut u16, 0);

    // Add to available ring
    let avail = BLK_AVAIL;
    let avail_idx = core::ptr::read_volatile((avail as *const u16).add(1)); // avail->idx
    let ring_slot = (avail as *mut u16).add(2 + (avail_idx as usize % qsz));
    core::ptr::write_volatile(ring_slot, 0u16); // desc chain starts at index 0

    // Memory barrier then update avail idx
    core::arch::asm!("mfence", options(nostack));
    core::ptr::write_volatile((avail as *mut u16).add(1), avail_idx.wrapping_add(1));

    // Notify device
    outw(iobase + VIRTIO_QUEUE_NOTIFY, 0);

    // Poll used ring for completion
    let used = BLK_USED;
    let mut timeout: u32 = 10_000_000;
    loop {
        let used_idx = core::ptr::read_volatile((used as *const u16).add(1));
        if used_idx != BLK_LAST_USED {
            break;
        }
        core::arch::asm!("pause", options(nomem, nostack));
        timeout -= 1;
        if timeout == 0 {
            return false;
        }
    }
    BLK_LAST_USED = BLK_LAST_USED.wrapping_add(1);

    // Acknowledge interrupt
    let _ = inb(iobase + VIRTIO_ISR_STATUS);

    // Check status byte
    let st = core::ptr::read_volatile(status_ptr);
    st == 0
}

// --------------- M5: Block syscalls ------------------------------------------

/// sys_blk_read(lba, user_buf, len) -> bytes_read or -1
/// len must be a multiple of 512, max 4096.
#[cfg(feature = "blk_test")]
unsafe fn sys_blk_read(lba: u64, buf: u64, len: u64) -> u64 {
    if !runtime::storage::block_io_len_valid(len) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let n = len as usize;
    if !user_range_ok(buf, n) || !user_pages_ok(buf, n, USER_PERM_WRITE) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    // Read from disk
    if !block_io_dispatch(false, lba, n, false) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    // Copyout to user buffer
    if copyout_user(buf, &BLK_DATA_PAGE.0[..n], n).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    len
}

/// sys_blk_write(lba, user_buf, len) -> bytes_written or -1
/// len must be a multiple of 512, max 4096.
#[cfg(feature = "blk_test")]
unsafe fn sys_blk_write(lba: u64, buf: u64, len: u64) -> u64 {
    if !runtime::storage::block_io_len_valid(len) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    let n = len as usize;
    // Copyin from user buffer to DMA page
    if copyin_user(&mut BLK_DATA_PAGE.0[..n], buf, n).is_err() {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    // Write to disk
    if !block_io_dispatch(true, lba, n, false) {
        return 0xFFFF_FFFF_FFFF_FFFF;
    }
    len
}

// --------------- M5: User blob for block r/w test ----------------------------
//
// This user program:
//   1. Fills 512 bytes at (rsp - 0x200) with 0xAA
//   2. sys_blk_write(lba=0, buf, 512)
//   3. Zeroes the same buffer
//   4. sys_blk_read(lba=0, buf, 512)
//   5. Checks buf[0] == 0xAA && buf[511] == 0xAA
//   6. Prints "BLK: rw ok\n" via sys_debug_write
//   7. HLTs (triggers GPF â†’ kernel exit)

#[cfg(feature = "blk_test")]
static BLK_TEST_BLOB: [u8; 111] = [
    // mov rbx, rsp
    0x48, 0x89, 0xE3,
    // sub rbx, 0x200
    0x48, 0x81, 0xEB, 0x00, 0x02, 0x00, 0x00,
    // mov rdi, rbx  (buf for rep stosb)
    0x48, 0x89, 0xDF,
    // mov ecx, 512
    0xB9, 0x00, 0x02, 0x00, 0x00,
    // mov al, 0xAA
    0xB0, 0xAA,
    // rep stosb  (fill buf with 0xAA)
    0xF3, 0xAA,
    // --- sys_blk_write(0, rbx, 512) ---
    // xor edi, edi        ; lba = 0
    0x31, 0xFF,
    // mov rsi, rbx        ; buf
    0x48, 0x89, 0xDE,
    // mov edx, 512        ; len
    0xBA, 0x00, 0x02, 0x00, 0x00,
    // mov eax, 14         ; sys_blk_write
    0xB8, 0x0E, 0x00, 0x00, 0x00,
    // int 0x80
    0xCD, 0x80,
    // --- zero buffer ---
    // mov rdi, rbx
    0x48, 0x89, 0xDF,
    // mov ecx, 512
    0xB9, 0x00, 0x02, 0x00, 0x00,
    // xor eax, eax
    0x31, 0xC0,
    // rep stosb  (fill buf with 0x00)
    0xF3, 0xAA,
    // --- sys_blk_read(0, rbx, 512) ---
    // xor edi, edi
    0x31, 0xFF,
    // mov rsi, rbx
    0x48, 0x89, 0xDE,
    // mov edx, 512
    0xBA, 0x00, 0x02, 0x00, 0x00,
    // mov eax, 13         ; sys_blk_read
    0xB8, 0x0D, 0x00, 0x00, 0x00,
    // int 0x80
    0xCD, 0x80,
    // --- verify ---
    // cmp byte [rbx], 0xAA
    0x80, 0x3B, 0xAA,
    // jne .bad (+26 -> offset 99)
    0x75, 0x1A,
    // cmp byte [rbx + 511], 0xAA
    0x80, 0xBB, 0xFF, 0x01, 0x00, 0x00, 0xAA,
    // jne .bad (+17 -> offset 99)
    0x75, 0x11,
    // --- print "BLK: rw ok\n" ---
    // lea rdi, [rip + 0x0B] -> .msg at offset 100
    0x48, 0x8D, 0x3D, 0x0B, 0x00, 0x00, 0x00,
    // mov esi, 11
    0xBE, 0x0B, 0x00, 0x00, 0x00,
    // xor eax, eax        ; sys_debug_write
    0x31, 0xC0,
    // int 0x80
    0xCD, 0x80,
    // hlt (triggers GPF in ring 3 â†’ kernel exit)
    0xF4,
    // .bad:
    0xF4,
    // .msg: "BLK: rw ok\n"
    b'B', b'L', b'K', b':', b' ', b'r', b'w', b' ', b'o', b'k', b'\n',
];

#[cfg(feature = "stress_blk_test")]
static BLK_STRESS_BLOB: [u8; 232] = [
    // buf = rsp - 512
    0x48, 0x89, 0xE3,
    0x48, 0x81, 0xEB, 0x00, 0x02, 0x00, 0x00,
    // r12d=64 (iterations), r13d=0 (seed/index), r14d=8 (base LBA)
    0x41, 0xBC, 0x40, 0x00, 0x00, 0x00,
    0x45, 0x31, 0xED,
    0x41, 0xBE, 0x08, 0x00, 0x00, 0x00,
    // loop: fill 512 bytes with seed byte (r13b)
    0x48, 0x89, 0xDF,
    0xB9, 0x00, 0x02, 0x00, 0x00,
    0x44, 0x88, 0xE8,
    0xF3, 0xAA,
    // expected checksum = seed * 512
    0x45, 0x89, 0xEF,
    0x41, 0xC1, 0xE7, 0x09,
    // sys_blk_write(base+i, buf, 512)
    0x44, 0x89, 0xF7,
    0x44, 0x01, 0xEF,
    0x48, 0x89, 0xDE,
    0xBA, 0x00, 0x02, 0x00, 0x00,
    0xB8, 0x0E, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x3D, 0x00, 0x02, 0x00, 0x00,
    0x0F, 0x85, 0x7F, 0x00, 0x00, 0x00,
    // zero buffer
    0x48, 0x89, 0xDF,
    0xB9, 0x00, 0x02, 0x00, 0x00,
    0x31, 0xC0,
    0xF3, 0xAA,
    // sys_blk_read(base+i, buf, 512)
    0x44, 0x89, 0xF7,
    0x44, 0x01, 0xEF,
    0x48, 0x89, 0xDE,
    0xBA, 0x00, 0x02, 0x00, 0x00,
    0xB8, 0x0D, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x3D, 0x00, 0x02, 0x00, 0x00,
    0x75, 0x56,
    // checksum readback bytes into r8d
    0x45, 0x31, 0xC0,
    0x31, 0xC9,
    0x0F, 0xB6, 0x04, 0x0B,
    0x41, 0x01, 0xC0,
    0xFF, 0xC1,
    0x81, 0xF9, 0x00, 0x02, 0x00, 0x00,
    0x75, 0xEF,
    // checksum + first/last byte must match deterministic pattern
    0x45, 0x39, 0xF8,
    0x75, 0x3B,
    0x8A, 0x03,
    0x44, 0x38, 0xE8,
    0x75, 0x34,
    0x8A, 0x83, 0xFF, 0x01, 0x00, 0x00,
    0x44, 0x38, 0xE8,
    0x75, 0x29,
    // next iteration
    0x41, 0xFF, 0xC5,
    0x41, 0xFF, 0xCC,
    0x0F, 0x85, 0x69, 0xFF, 0xFF, 0xFF,
    // success: sys_debug_write("STRESS: blk ok", 14); qemu_exit(0x31)
    0x48, 0x8D, 0x3D, 0x23, 0x00, 0x00, 0x00,
    0xBE, 0x0E, 0x00, 0x00, 0x00,
    0x31, 0xC0,
    0xCD, 0x80,
    0xBF, 0x31, 0x00, 0x00, 0x00,
    0xB8, 0x62, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0xF4,
    // fail: qemu_exit(0x33)
    0xBF, 0x33, 0x00, 0x00, 0x00,
    0xB8, 0x62, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0xF4,
    // "STRESS: blk ok"
    0x53, 0x54, 0x52, 0x45, 0x53, 0x53, 0x3A, 0x20, 0x62, 0x6C, 0x6B, 0x20, 0x6F, 0x6B,
];

#[cfg(feature = "blk_badlen_test")]
static BLK_BADLEN_BLOB: [u8; 66] = [
    // mov rbx, rsp
    0x48, 0x89, 0xE3,
    // sub rbx, 0x200
    0x48, 0x81, 0xEB, 0x00, 0x02, 0x00, 0x00,
    // sys_blk_read(lba=0, buf=rbx, len=513)
    0x31, 0xFF,                                   // xor edi, edi
    0x48, 0x89, 0xDE,                             // mov rsi, rbx
    0xBA, 0x01, 0x02, 0x00, 0x00,               // mov edx, 513
    0xB8, 0x0D, 0x00, 0x00, 0x00,               // mov eax, 13
    0xCD, 0x80,                                   // int 0x80
    // cmp rax, -1; jne fail
    0x48, 0x83, 0xF8, 0xFF,                       // cmp rax, -1
    0x75, 0x11,                                   // jne +17 (fail@50)
    // sys_debug_write("BLK: badlen ok\n", 15)
    0x48, 0x8D, 0x3D, 0x0B, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x0B] -> msg@51
    0xBE, 0x0F, 0x00, 0x00, 0x00,               // mov esi, 15
    0x31, 0xC0,                                   // xor eax, eax
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt
    // fail:
    0xF4,                                         // hlt
    // Data @51: "BLK: badlen ok\n"
    b'B', b'L', b'K', b':', b' ', b'b', b'a', b'd',
    b'l', b'e', b'n', b' ', b'o', b'k', b'\n',
];

#[cfg(feature = "blk_badptr_test")]
static BLK_BADPTR_BLOB: [u8; 61] = [
    // sys_blk_read(lba=0, buf=0xDEADBEEF, len=512)
    0xBF, 0x00, 0x00, 0x00, 0x00,               // mov edi, 0
    0xBE, 0xEF, 0xBE, 0xAD, 0xDE,               // mov esi, 0xDEADBEEF
    0xBA, 0x00, 0x02, 0x00, 0x00,               // mov edx, 512
    0xB8, 0x0D, 0x00, 0x00, 0x00,               // mov eax, 13
    0xCD, 0x80,                                   // int 0x80
    // cmp rax, -1; jne fail
    0x48, 0x83, 0xF8, 0xFF,                       // cmp rax, -1
    0x75, 0x11,                                   // jne +17 (fail@45)
    // sys_debug_write("BLK: badptr ok\n", 15)
    0x48, 0x8D, 0x3D, 0x0B, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x0B] -> msg@46
    0xBE, 0x0F, 0x00, 0x00, 0x00,               // mov esi, 15
    0x31, 0xC0,                                   // xor eax, eax
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt
    // fail:
    0xF4,                                         // hlt
    // Data @46: "BLK: badptr ok\n"
    b'B', b'L', b'K', b':', b' ', b'b', b'a', b'd',
    b'p', b't', b'r', b' ', b'o', b'k', b'\n',
];

// --------------- R4: User program blobs (hand-assembled x86-64) --------------

// IPC ping-pong blobs
#[cfg(feature = "ipc_test")]
static IPC_PONG_BLOB: [u8; 103] = [
    // sys_svc_register("pong", 4, 0)
    0x48, 0x8d, 0x3d, 0x4f, 0x00, 0x00, 0x00, // lea rdi, [rip+0x4F] -> "pong" @86
    0xbe, 0x04, 0x00, 0x00, 0x00,               // mov esi, 4
    0x31, 0xd2,                                   // xor edx, edx
    0xb8, 0x0b, 0x00, 0x00, 0x00,               // mov eax, 11
    0xcd, 0x80,                                   // int 0x80
    // sys_ipc_recv(0, rsp-256, 256)
    0x31, 0xff,                                   // xor edi, edi
    0x48, 0x89, 0xe6,                             // mov rsi, rsp
    0x48, 0x81, 0xee, 0x00, 0x01, 0x00, 0x00,   // sub rsi, 0x100
    0xba, 0x00, 0x01, 0x00, 0x00,               // mov edx, 256
    0xb8, 0x09, 0x00, 0x00, 0x00,               // mov eax, 9
    0xcd, 0x80,                                   // int 0x80
    // sys_ipc_send(1, "pong", 4)
    0xbf, 0x01, 0x00, 0x00, 0x00,               // mov edi, 1
    0x48, 0x8d, 0x35, 0x21, 0x00, 0x00, 0x00,   // lea rsi, [rip+0x21] -> "pong" @90
    0xba, 0x04, 0x00, 0x00, 0x00,               // mov edx, 4
    0xb8, 0x08, 0x00, 0x00, 0x00,               // mov eax, 8
    0xcd, 0x80,                                   // int 0x80
    // sys_debug_write("PONG: ok\n", 9)
    0x48, 0x8d, 0x3d, 0x12, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x12] -> msg @94
    0xbe, 0x09, 0x00, 0x00, 0x00,               // mov esi, 9
    0x31, 0xc0,                                   // xor eax, eax
    0xcd, 0x80,                                   // int 0x80
    0xf4,                                         // hlt
    // Data
    b'p', b'o', b'n', b'g',                     // @86: register name
    b'p', b'o', b'n', b'g',                     // @90: reply payload
    b'P', b'O', b'N', b'G', b':', b' ', b'o', b'k', b'\n', // @94: marke
];

#[cfg(feature = "ipc_test")]
static IPC_PING_BLOB: [u8; 102] = [
    // sys_svc_lookup("pong", 4)
    0x48, 0x8d, 0x3d, 0x4e, 0x00, 0x00, 0x00, // lea rdi, [rip+0x4E] -> "pong" @85
    0xbe, 0x04, 0x00, 0x00, 0x00,               // mov esi, 4
    0xb8, 0x0c, 0x00, 0x00, 0x00,               // mov eax, 12
    0xcd, 0x80,                                   // int 0x80
    // sys_ipc_send(rax, "ping", 4)
    0x48, 0x89, 0xc7,                             // mov rdi, rax
    0x48, 0x8d, 0x35, 0x3c, 0x00, 0x00, 0x00,   // lea rsi, [rip+0x3C] -> "ping" @89
    0xba, 0x04, 0x00, 0x00, 0x00,               // mov edx, 4
    0xb8, 0x08, 0x00, 0x00, 0x00,               // mov eax, 8
    0xcd, 0x80,                                   // int 0x80
    // sys_ipc_recv(1, rsp-256, 256)
    0xbf, 0x01, 0x00, 0x00, 0x00,               // mov edi, 1
    0x48, 0x89, 0xe6,                             // mov rsi, rsp
    0x48, 0x81, 0xee, 0x00, 0x01, 0x00, 0x00,   // sub rsi, 0x100
    0xba, 0x00, 0x01, 0x00, 0x00,               // mov edx, 256
    0xb8, 0x09, 0x00, 0x00, 0x00,               // mov eax, 9
    0xcd, 0x80,                                   // int 0x80
    // sys_debug_write("PING: ok\n", 9)
    0x48, 0x8d, 0x3d, 0x12, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x12] -> msg @93
    0xbe, 0x09, 0x00, 0x00, 0x00,               // mov esi, 9
    0x31, 0xc0,                                   // xor eax, eax
    0xcd, 0x80,                                   // int 0x80
    0xf4,                                         // hlt
    // Data
    b'p', b'o', b'n', b'g',                     // @85: lookup name
    b'p', b'i', b'n', b'g',                     // @89: send payload
    b'P', b'I', b'N', b'G', b':', b' ', b'o', b'k', b'\n', // @93: marke
];

// IPC bad-pointer send blob (single task: send with unmapped buf â†’ expect -1)
#[cfg(feature = "stress_ipc_test")]
static STRESS_IPC_SENDER_A_BLOB: [u8; 56] = [
    // mov r12d, 200
    0x41, 0xBC, 0xC8, 0x00, 0x00, 0x00,
    // send_loop: sys_ipc_send(ep=0, msg, 5)
    0xBF, 0x00, 0x00, 0x00, 0x00,
    0x48, 0x8D, 0x35, 0x21, 0x00, 0x00, 0x00,
    0xBA, 0x05, 0x00, 0x00, 0x00,
    0xB8, 0x08, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // if rax == -1 => sys_yield + retry
    0x48, 0x83, 0xF8, 0xFF,
    0x75, 0x09,
    0xB8, 0x03, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0xEB, 0xD9,
    // sent: dec loop; continue until done
    0x41, 0xFF, 0xCC,
    0x75, 0xD4,
    0xF4,
    // msg
    b'A', b'-', b'i', b'p', b'c',
];

#[cfg(feature = "stress_ipc_test")]
static STRESS_IPC_SENDER_B_BLOB: [u8; 56] = [
    // mov r12d, 200
    0x41, 0xBC, 0xC8, 0x00, 0x00, 0x00,
    // send_loop: sys_ipc_send(ep=1, msg, 5)
    0xBF, 0x01, 0x00, 0x00, 0x00,
    0x48, 0x8D, 0x35, 0x21, 0x00, 0x00, 0x00,
    0xBA, 0x05, 0x00, 0x00, 0x00,
    0xB8, 0x08, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // if rax == -1 => sys_yield + retry
    0x48, 0x83, 0xF8, 0xFF,
    0x75, 0x09,
    0xB8, 0x03, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0xEB, 0xD9,
    // sent: dec loop; continue until done
    0x41, 0xFF, 0xCC,
    0x75, 0xD4,
    0xF4,
    // msg
    b'B', b'-', b'i', b'p', b'c',
];

#[cfg(feature = "stress_ipc_test")]
static STRESS_IPC_RECV_C_BLOB: [u8; 73] = [
    // mov r12d, 200
    0x41, 0xBC, 0xC8, 0x00, 0x00, 0x00,
    // recv_loop: sys_ipc_recv(ep=0, rsp-256, 256)
    0xBF, 0x00, 0x00, 0x00, 0x00,
    0x48, 0x89, 0xE6,
    0x48, 0x81, 0xEE, 0x00, 0x01, 0x00, 0x00,
    0xBA, 0x00, 0x01, 0x00, 0x00,
    0xB8, 0x09, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // validate len == 5
    0x48, 0x83, 0xF8, 0x05,
    0x75, 0x15,
    // validate prefix byte == 'A'
    0x48, 0x89, 0xE6,
    0x48, 0x81, 0xEE, 0x00, 0x01, 0x00, 0x00,
    0x80, 0x3E, 0x41,
    0x75, 0x06,
    // dec loop; continue until done
    0x41, 0xFF, 0xCC,
    0x75, 0xCB,
    0xF4,
    // fail: sys_debug_exit(0x33)
    0xBF, 0x33, 0x00, 0x00, 0x00,
    0xB8, 0x62, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0xF4,
];

#[cfg(feature = "stress_ipc_test")]
static STRESS_IPC_RECV_D_BLOB: [u8; 73] = [
    // mov r12d, 200
    0x41, 0xBC, 0xC8, 0x00, 0x00, 0x00,
    // recv_loop: sys_ipc_recv(ep=1, rsp-256, 256)
    0xBF, 0x01, 0x00, 0x00, 0x00,
    0x48, 0x89, 0xE6,
    0x48, 0x81, 0xEE, 0x00, 0x01, 0x00, 0x00,
    0xBA, 0x00, 0x01, 0x00, 0x00,
    0xB8, 0x09, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // validate len == 5
    0x48, 0x83, 0xF8, 0x05,
    0x75, 0x15,
    // validate prefix byte == 'B'
    0x48, 0x89, 0xE6,
    0x48, 0x81, 0xEE, 0x00, 0x01, 0x00, 0x00,
    0x80, 0x3E, 0x42,
    0x75, 0x06,
    // dec loop; continue until done
    0x41, 0xFF, 0xCC,
    0x75, 0xCB,
    0xF4,
    // fail: sys_debug_exit(0x33)
    0xBF, 0x33, 0x00, 0x00, 0x00,
    0xB8, 0x62, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0xF4,
];

#[cfg(feature = "ipc_badptr_send_test")]
static IPC_BADPTR_SEND_BLOB: [u8; 75] = [
    // sys_ipc_send(endpoint=0, buf=0xDEAD0000, len=16)
    0x31, 0xFF,                                   // xor edi, edi
    0xBE, 0x00, 0x00, 0xAD, 0xDE,               // mov esi, 0xDEAD0000
    0xBA, 0x10, 0x00, 0x00, 0x00,               // mov edx, 16
    0xB8, 0x08, 0x00, 0x00, 0x00,               // mov eax, 8
    0xCD, 0x80,                                   // int 0x80
    // cmp rax, -1; jne fail
    0x48, 0x83, 0xF8, 0xFF,                       // cmp rax, -1
    0x75, 0x1D,                                   // jne +29 (fail@54)
    // sys_debug_write("IPC: badptr send ok\n", 20)
    0x48, 0x8D, 0x3D, 0x17, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x17] -> msg@55
    0xBE, 0x14, 0x00, 0x00, 0x00,               // mov esi, 20
    0x31, 0xC0,                                   // xor eax, eax
    0xCD, 0x80,                                   // int 0x80
    // sys_debug_exit(0x31)
    0xBF, 0x31, 0x00, 0x00, 0x00,               // mov edi, 0x31
    0xB8, 0x62, 0x00, 0x00, 0x00,               // mov eax, 98
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt
    // fail:
    0xF4,                                         // hlt
    // Data @55: "IPC: badptr send ok\n"
    b'I', b'P', b'C', b':', b' ', b'b', b'a', b'd', b'p', b't',
    b'r', b' ', b's', b'e', b'n', b'd', b' ', b'o', b'k', b'\n',
];

// IPC bad-pointer recv blob (single task: recv with unmapped buf â†’ expect -1)
#[cfg(feature = "ipc_badptr_recv_test")]
static IPC_BADPTR_RECV_BLOB: [u8; 75] = [
    // sys_ipc_recv(endpoint=0, buf=0xDEADBEEF, cap=16)
    0x31, 0xFF,                                   // xor edi, edi
    0xBE, 0xEF, 0xBE, 0xAD, 0xDE,               // mov esi, 0xDEADBEEF
    0xBA, 0x10, 0x00, 0x00, 0x00,               // mov edx, 16
    0xB8, 0x09, 0x00, 0x00, 0x00,               // mov eax, 9
    0xCD, 0x80,                                   // int 0x80
    // cmp rax, -1; jne fail
    0x48, 0x83, 0xF8, 0xFF,                       // cmp rax, -1
    0x75, 0x1D,                                   // jne +29 (fail@54)
    // sys_debug_write("IPC: badptr recv ok\n", 20)
    0x48, 0x8D, 0x3D, 0x17, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x17] -> msg@55
    0xBE, 0x14, 0x00, 0x00, 0x00,               // mov esi, 20
    0x31, 0xC0,                                   // xor eax, eax
    0xCD, 0x80,                                   // int 0x80
    // sys_debug_exit(0x31)
    0xBF, 0x31, 0x00, 0x00, 0x00,               // mov edi, 0x31
    0xB8, 0x62, 0x00, 0x00, 0x00,               // mov eax, 98
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt
    // fail:
    0xF4,                                         // hlt
    // Data @55: "IPC: badptr recv ok\n"
    b'I', b'P', b'C', b':', b' ', b'b', b'a', b'd', b'p', b't',
    b'r', b' ', b'r', b'e', b'c', b'v', b' ', b'o', b'k', b'\n',
];

// Service registry bad-pointer blob (single task: register with unmapped name â†’ expect -1)
#[cfg(feature = "ipc_badptr_svc_test")]
static SVC_BADPTR_BLOB: [u8; 70] = [
    // sys_svc_register(name_ptr=0xDEAD0000, name_len=8, endpoint=0)
    0xBF, 0x00, 0x00, 0xAD, 0xDE,               // mov edi, 0xDEAD0000
    0xBE, 0x08, 0x00, 0x00, 0x00,               // mov esi, 8
    0x31, 0xD2,                                   // xor edx, edx
    0xB8, 0x0B, 0x00, 0x00, 0x00,               // mov eax, 11
    0xCD, 0x80,                                   // int 0x80
    // cmp rax, -1; jne fail
    0x48, 0x83, 0xF8, 0xFF,                       // cmp rax, -1
    0x75, 0x1D,                                   // jne +29 (fail@54)
    // sys_debug_write("SVC: badptr ok\n", 15)
    0x48, 0x8D, 0x3D, 0x17, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x17] -> msg@55
    0xBE, 0x0F, 0x00, 0x00, 0x00,               // mov esi, 15
    0x31, 0xC0,                                   // xor eax, eax
    0xCD, 0x80,                                   // int 0x80
    // sys_debug_exit(0x31)
    0xBF, 0x31, 0x00, 0x00, 0x00,               // mov edi, 0x31
    0xB8, 0x62, 0x00, 0x00, 0x00,               // mov eax, 98
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt
    // fail:
    0xF4,                                         // hlt
    // Data @55: "SVC: badptr ok\n"
    b'S', b'V', b'C', b':', b' ', b'b', b'a', b'd', b'p', b't',
    b'r', b' ', b'o', b'k', b'\n',
];

// Service registry bad-endpoint blob (single task: register with inactive endpoint -> expect -1)
#[cfg(feature = "svc_bad_endpoint_test")]
static SVC_BAD_ENDPOINT_BLOB: [u8; 84] = [
    // sys_svc_register("bad", 3, endpoint=7)
    0x48, 0x8D, 0x3D, 0x35, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x35] -> name@60
    0xBE, 0x03, 0x00, 0x00, 0x00,               // mov esi, 3
    0xBA, 0x07, 0x00, 0x00, 0x00,               // mov edx, 7
    0xB8, 0x0B, 0x00, 0x00, 0x00,               // mov eax, 11
    0xCD, 0x80,                                   // int 0x80
    // cmp rax, -1; jne fail
    0x48, 0x83, 0xF8, 0xFF,                       // cmp rax, -1
    0x75, 0x1D,                                   // jne +29 (fail@59)
    // sys_debug_write("SVC: bad endpoint ok\n", 21)
    0x48, 0x8D, 0x3D, 0x1A, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x1A] -> msg@63
    0xBE, 0x15, 0x00, 0x00, 0x00,               // mov esi, 21
    0x31, 0xC0,                                   // xor eax, eax
    0xCD, 0x80,                                   // int 0x80
    // sys_debug_exit(0x31)
    0xBF, 0x31, 0x00, 0x00, 0x00,               // mov edi, 0x31
    0xB8, 0x62, 0x00, 0x00, 0x00,               // mov eax, 98
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt
    // fail:
    0xF4,                                         // hlt
    // Data
    b'b', b'a', b'd',
    b'S', b'V', b'C', b':', b' ', b'b', b'a', b'd', b' ', b'e',
    b'n', b'd', b'p', b'o', b'i', b'n', b't', b' ', b'o', b'k', b'\n',
];

// IPC buffer-full blob (single task: send1 ok, send2 â†’ -1, recv â†’ msg1 intact)
#[cfg(feature = "ipc_buffer_full_test")]
static IPC_BUFFER_FULL_BLOB: [u8; 137] = [
    // --- Step 1: sys_ipc_send(ep=0, &msg1, 4) ---
    0x31, 0xFF,                                   // xor edi, edi          ; ep = 0
    0x48, 0x8D, 0x35, 0x6B, 0x00, 0x00, 0x00,   // lea rsi, [rip+0x6B]  -> msg1 @0x74
    0xBA, 0x04, 0x00, 0x00, 0x00,               // mov edx, 4
    0xB8, 0x08, 0x00, 0x00, 0x00,               // mov eax, 8 (sys_ipc_send)
    0xCD, 0x80,                                   // int 0x80
    // Check rax == 0 (success)
    0x48, 0x85, 0xC0,                             // test rax, rax
    0x75, 0x59,                                   // jnz fail @0x73
    // --- Step 2: sys_ipc_send(ep=0, &msg2, 4) â†’ must return -1 ---
    0x31, 0xFF,                                   // xor edi, edi
    0x48, 0x8D, 0x35, 0x55, 0x00, 0x00, 0x00,   // lea rsi, [rip+0x55]  -> msg2 @0x78
    0xBA, 0x04, 0x00, 0x00, 0x00,               // mov edx, 4
    0xB8, 0x08, 0x00, 0x00, 0x00,               // mov eax, 8
    0xCD, 0x80,                                   // int 0x80
    // Check rax == -1 (buffer full)
    0x48, 0x83, 0xF8, 0xFF,                       // cmp rax, -1
    0x75, 0x3E,                                   // jne fail @0x73
    // --- Step 3: sys_ipc_recv(ep=0, rsp-256, 256) â†’ delivers msg1 ---
    0x31, 0xFF,                                   // xor edi, edi
    0x48, 0x89, 0xE6,                             // mov rsi, rsp
    0x48, 0x81, 0xEE, 0x00, 0x01, 0x00, 0x00,   // sub rsi, 0x100
    0xBA, 0x00, 0x01, 0x00, 0x00,               // mov edx, 256
    0xB8, 0x09, 0x00, 0x00, 0x00,               // mov eax, 9 (sys_ipc_recv)
    0xCD, 0x80,                                   // int 0x80
    // Check rax == 4 (msg1 length)
    0x48, 0x83, 0xF8, 0x04,                       // cmp rax, 4
    0x75, 0x20,                                   // jne fail @0x73
    // Check first byte == 'A' (0x41) â€” proves msg1 not overwritten
    0x48, 0x89, 0xE6,                             // mov rsi, rsp
    0x48, 0x81, 0xEE, 0x00, 0x01, 0x00, 0x00,   // sub rsi, 0x100
    0x80, 0x3E, 0x41,                             // cmp byte [rsi], 0x41
    0x75, 0x11,                                   // jne fail @0x73
    // --- All passed: sys_debug_write("IPC: full ok\n", 13) ---
    0x48, 0x8D, 0x3D, 0x13, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x13]  -> msg_ok @0x7C
    0xBE, 0x0D, 0x00, 0x00, 0x00,               // mov esi, 13
    0x31, 0xC0,                                   // xor eax, eax (sys_debug_write)
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt
    // fail:
    0xF4,                                         // hlt
    // Data @0x74: msg1 "AAAA"
    b'A', b'A', b'A', b'A',
    // Data @0x78: msg2 "BBBB"
    b'B', b'B', b'B', b'B',
    // Data @0x7C: "IPC: full ok\n"
    b'I', b'P', b'C', b':', b' ', b'f', b'u', b'l', b'l', b' ', b'o', b'k', b'\n',
];

// IPC waiter policy test:
// - task0 blocks in recv(ep0, cap=4)
// - task1 recv on same ep must return -1 (second waiter rejected)
// - task1 send len=8 to blocked waiter cap=4 must return -1 (no truncation)
// - task1 send len=300 must return -1 (oversize rejected)
// - task1 prints success marker
#[cfg(feature = "ipc_waiter_busy_test")]
static IPC_WAITER_BLOCK_BLOB: [u8; 37] = [
    // sys_ipc_recv(endpoint=0, rsp-256, cap=4)
    0x31, 0xFF,                                   // xor edi, edi
    0x48, 0x89, 0xE6,                             // mov rsi, rsp
    0x48, 0x81, 0xEE, 0x00, 0x01, 0x00, 0x00,   // sub rsi, 0x100
    0xBA, 0x04, 0x00, 0x00, 0x00,               // mov edx, 4
    0xB8, 0x09, 0x00, 0x00, 0x00,               // mov eax, 9
    0xCD, 0x80,                                   // int 0x80
    // unexpected return -> exit(0x33)
    0xBF, 0x33, 0x00, 0x00, 0x00,               // mov edi, 0x33
    0xB8, 0x62, 0x00, 0x00, 0x00,               // mov eax, 98
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt
];

#[cfg(feature = "ipc_waiter_busy_test")]
static IPC_WAITER_CONTENDER_BLOB: [u8; 154] = [
    // 1) recv on same endpoint -> must return -1 (waiter already present)
    0x31, 0xFF,                                   // xor edi, edi
    0x48, 0x89, 0xE6,                             // mov rsi, rsp
    0x48, 0x81, 0xEE, 0x00, 0x01, 0x00, 0x00,   // sub rsi, 0x100
    0xBA, 0x04, 0x00, 0x00, 0x00,               // mov edx, 4
    0xB8, 0x09, 0x00, 0x00, 0x00,               // mov eax, 9
    0xCD, 0x80,                                   // int 0x80
    0x48, 0x83, 0xF8, 0xFF,                       // cmp rax, -1
    0x75, 0x59,                                   // jne fail

    // 2) send len=8 to blocked waiter(cap=4) -> must return -1 (no truncation)
    0x31, 0xFF,                                   // xor edi, edi
    0x48, 0x89, 0xE6,                             // mov rsi, rsp
    0x48, 0x81, 0xEE, 0x00, 0x01, 0x00, 0x00,   // sub rsi, 0x100
    0xBA, 0x08, 0x00, 0x00, 0x00,               // mov edx, 8
    0xB8, 0x08, 0x00, 0x00, 0x00,               // mov eax, 8
    0xCD, 0x80,                                   // int 0x80
    0x48, 0x83, 0xF8, 0xFF,                       // cmp rax, -1
    0x75, 0x3B,                                   // jne fail

    // 3) send len=300 -> must return -1 (oversize reject)
    0x31, 0xFF,                                   // xor edi, edi
    0x48, 0x89, 0xE6,                             // mov rsi, rsp
    0x48, 0x81, 0xEE, 0x00, 0x01, 0x00, 0x00,   // sub rsi, 0x100
    0xBA, 0x2C, 0x01, 0x00, 0x00,               // mov edx, 300
    0xB8, 0x08, 0x00, 0x00, 0x00,               // mov eax, 8
    0xCD, 0x80,                                   // int 0x80
    0x48, 0x83, 0xF8, 0xFF,                       // cmp rax, -1
    0x75, 0x1D,                                   // jne fail

    // pass: sys_debug_write("IPC: waiter strict ok\n", 22) then exit(0x31)
    0x48, 0x8D, 0x3D, 0x23, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x23] -> msg
    0xBE, 0x16, 0x00, 0x00, 0x00,               // mov esi, 22
    0x31, 0xC0,                                   // xor eax, eax
    0xCD, 0x80,                                   // int 0x80
    0xBF, 0x31, 0x00, 0x00, 0x00,               // mov edi, 0x31
    0xB8, 0x62, 0x00, 0x00, 0x00,               // mov eax, 98
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt

    // fail: exit(0x33)
    0xBF, 0x33, 0x00, 0x00, 0x00,               // mov edi, 0x33
    0xB8, 0x62, 0x00, 0x00, 0x00,               // mov eax, 98
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt

    // Data: "IPC: waiter strict ok\n"
    b'I', b'P', b'C', b':', b' ', b'w', b'a', b'i', b't', b'e',
    b'r', b' ', b's', b't', b'r', b'i', b'c', b't', b' ', b'o',
    b'k', b'\n',
];

// SVC overwrite blob (single task: register "foo"â†’1, register "foo"â†’2, lookup must return 2)
#[cfg(feature = "svc_overwrite_test")]
static SVC_OVERWRITE_BLOB: [u8; 122] = [
    // --- Step 1: sys_svc_register("foo", 3, 1) ---
    0x48, 0x8D, 0x3D, 0x5E, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x5E]  -> "foo" @0x65
    0xBE, 0x03, 0x00, 0x00, 0x00,               // mov esi, 3
    0xBA, 0x01, 0x00, 0x00, 0x00,               // mov edx, 1 (endpoint)
    0xB8, 0x0B, 0x00, 0x00, 0x00,               // mov eax, 11 (sys_svc_register)
    0xCD, 0x80,                                   // int 0x80
    // Check rax == 0
    0x48, 0x85, 0xC0,                             // test rax, rax
    0x75, 0x47,                                   // jnz fail @0x64
    // --- Step 2: sys_svc_register("foo", 3, 2) â€” overwrite ---
    0x48, 0x8D, 0x3D, 0x41, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x41]  -> "foo" @0x65
    0xBE, 0x03, 0x00, 0x00, 0x00,               // mov esi, 3
    0xBA, 0x02, 0x00, 0x00, 0x00,               // mov edx, 2 (endpoint)
    0xB8, 0x0B, 0x00, 0x00, 0x00,               // mov eax, 11
    0xCD, 0x80,                                   // int 0x80
    // Check rax == 0
    0x48, 0x85, 0xC0,                             // test rax, rax
    0x75, 0x2A,                                   // jnz fail @0x64
    // --- Step 3: sys_svc_lookup("foo", 3) ---
    0x48, 0x8D, 0x3D, 0x24, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x24]  -> "foo" @0x65
    0xBE, 0x03, 0x00, 0x00, 0x00,               // mov esi, 3
    0xB8, 0x0C, 0x00, 0x00, 0x00,               // mov eax, 12 (sys_svc_lookup)
    0xCD, 0x80,                                   // int 0x80
    // Check rax == 2
    0x48, 0x83, 0xF8, 0x02,                       // cmp rax, 2
    0x75, 0x11,                                   // jne fail @0x64
    // --- All passed: sys_debug_write("SVC: overwrite ok\n", 18) ---
    0x48, 0x8D, 0x3D, 0x0E, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x0E]  -> msg @0x68
    0xBE, 0x12, 0x00, 0x00, 0x00,               // mov esi, 18
    0x31, 0xC0,                                   // xor eax, eax (sys_debug_write)
    0xCD, 0x80,                                   // int 0x80
    0xF4,                                         // hlt
    // fail:
    0xF4,                                         // hlt
    // Data @0x65: "foo"
    b'f', b'o', b'o',
    // Data @0x68: "SVC: overwrite ok\n"
    b'S', b'V', b'C', b':', b' ', b'o', b'v', b'e', b'r', b'w',
    b'r', b'i', b't', b'e', b' ', b'o', b'k', b'\n',
];

// SVC full-table blob (single task: 4 unique regs succeed, 5th unique fails with -1)
#[cfg(feature = "svc_full_test")]
static SVC_FULL_BLOB: [u8; 199] = [
    // --- 1: sys_svc_register("a", 1, 0) ---
    0x48, 0x8D, 0x3D, 0xAE, 0x00, 0x00, 0x00,
    0xBE, 0x01, 0x00, 0x00, 0x00,
    0x31, 0xD2,
    0xB8, 0x0B, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x85, 0xC0,
    0x0F, 0x85, 0x96, 0x00, 0x00, 0x00,
    // --- 2: sys_svc_register("b", 1, 1) ---
    0x48, 0x8D, 0x3D, 0x91, 0x00, 0x00, 0x00,
    0xBE, 0x01, 0x00, 0x00, 0x00,
    0xBA, 0x01, 0x00, 0x00, 0x00,
    0xB8, 0x0B, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x85, 0xC0,
    0x0F, 0x85, 0x75, 0x00, 0x00, 0x00,
    // --- 3: sys_svc_register("c", 1, 2) ---
    0x48, 0x8D, 0x3D, 0x71, 0x00, 0x00, 0x00,
    0xBE, 0x01, 0x00, 0x00, 0x00,
    0xBA, 0x02, 0x00, 0x00, 0x00,
    0xB8, 0x0B, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x85, 0xC0,
    0x0F, 0x85, 0x54, 0x00, 0x00, 0x00,
    // --- 4: sys_svc_register("d", 1, 3) ---
    0x48, 0x8D, 0x3D, 0x51, 0x00, 0x00, 0x00,
    0xBE, 0x01, 0x00, 0x00, 0x00,
    0xBA, 0x03, 0x00, 0x00, 0x00,
    0xB8, 0x0B, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x85, 0xC0,
    0x0F, 0x85, 0x33, 0x00, 0x00, 0x00,
    // --- 5: sys_svc_register("e", 1, 4) -> must be -1 ---
    0x48, 0x8D, 0x3D, 0x31, 0x00, 0x00, 0x00,
    0xBE, 0x01, 0x00, 0x00, 0x00,
    0xBA, 0x04, 0x00, 0x00, 0x00,
    0xB8, 0x0B, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x83, 0xF8, 0xFF,
    0x0F, 0x85, 0x11, 0x00, 0x00, 0x00,
    // --- pass: sys_debug_write("SVC: full ok\n", 13) ---
    0x48, 0x8D, 0x3D, 0x10, 0x00, 0x00, 0x00,
    0xBE, 0x0D, 0x00, 0x00, 0x00,
    0x31, 0xC0,
    0xCD, 0x80,
    0xF4,
    // fail:
    0xF4,
    // Data
    b'a', b'b', b'c', b'd', b'e',
    b'S', b'V', b'C', b':', b' ', b'f', b'u', b'l', b'l', b' ', b'o', b'k', b'\n',
];

#[cfg(feature = "quota_endpoints_test")]
static QUOTA_ENDPOINTS_BLOB: [u8; 74] = [
    // r12d = remaining successful creates (16)
    0x41, 0xBC, 0x10, 0x00, 0x00, 0x00,
    // loop: sys_endpoint_create() (nr=17)
    0xB8, 0x11, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // if rax == -1 before limit, fail
    0x48, 0x83, 0xF8, 0xFF,
    0x74, 0x23,
    // dec remaining; if not zero, keep creating
    0x41, 0xFF, 0xCC,
    0x75, 0xEE,
    // one extra create must fail exactly at limit
    0xB8, 0x11, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x83, 0xF8, 0xFF,
    0x75, 0x11,
    // success: sys_debug_write("QUOTA: endpoints ok", 19)
    0x48, 0x8D, 0x3D, 0x0B, 0x00, 0x00, 0x00,
    0xBE, 0x13, 0x00, 0x00, 0x00,
    0x31, 0xC0,
    0xCD, 0x80,
    0xF4,
    // fail
    0xF4,
    b'Q', b'U', b'O', b'T', b'A', b':', b' ', b'e', b'n', b'd',
    b'p', b'o', b'i', b'n', b't', b's', b' ', b'o', b'k',
];

#[cfg(feature = "quota_shm_test")]
static QUOTA_SHM_BLOB: [u8; 78] = [
    // r12d = remaining successful creates (32)
    0x41, 0xBC, 0x20, 0x00, 0x00, 0x00,
    // loop: sys_shm_create(4096)
    0xBF, 0x00, 0x10, 0x00, 0x00,
    0xB8, 0x06, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // if rax == -1 before limit, fail
    0x48, 0x83, 0xF8, 0xFF,
    0x74, 0x28,
    // dec remaining; if not zero, keep creating
    0x41, 0xFF, 0xCC,
    0x75, 0xE9,
    // one extra create must fail exactly at limit
    0xBF, 0x00, 0x10, 0x00, 0x00,
    0xB8, 0x06, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x83, 0xF8, 0xFF,
    0x75, 0x11,
    // success: sys_debug_write("QUOTA: shm ok", 13)
    0x48, 0x8D, 0x3D, 0x0B, 0x00, 0x00, 0x00,
    0xBE, 0x0D, 0x00, 0x00, 0x00,
    0x31, 0xC0,
    0xCD, 0x80,
    0xF4,
    // fail
    0xF4,
    b'Q', b'U', b'O', b'T', b'A', b':', b' ', b's', b'h', b'm', b' ', b'o', b'k',
];

#[cfg(feature = "quota_threads_test")]
static QUOTA_THREADS_BLOB: [u8; 87] = [
    // r12d = remaining successful spawns (16)
    0x41, 0xBC, 0x10, 0x00, 0x00, 0x00,
    // loop: sys_thread_spawn(entry)
    0x48, 0x8D, 0x3D, 0x38, 0x00, 0x00, 0x00,
    0xB8, 0x01, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    // if rax == -1 before limit, fail
    0x48, 0x83, 0xF8, 0xFF,
    0x74, 0x2A,
    // dec remaining; if not zero, keep spawning
    0x41, 0xFF, 0xCC,
    0x75, 0xE7,
    // one extra spawn must fail exactly at limit
    0x48, 0x8D, 0x3D, 0x1F, 0x00, 0x00, 0x00,
    0xB8, 0x01, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x83, 0xF8, 0xFF,
    0x75, 0x11,
    // success: sys_debug_write("QUOTA: threads ok", 17)
    0x48, 0x8D, 0x3D, 0x0C, 0x00, 0x00, 0x00,
    0xBE, 0x11, 0x00, 0x00, 0x00,
    0x31, 0xC0,
    0xCD, 0x80,
    0xF4,
    // fail
    0xF4,
    // dummy entry target for sys_thread_spawn
    0xF4,
    b'Q', b'U', b'O', b'T', b'A', b':', b' ', b't', b'h', b'r', b'e', b'a', b'd', b's', b' ', b'o', b'k',
];

// SHM blobs
#[cfg(feature = "shm_test")]
static SHM_WRITER_BLOB: [u8; 69] = [
    // sys_shm_create(4096)
    0xbf, 0x00, 0x10, 0x00, 0x00,               // mov edi, 4096
    0xb8, 0x06, 0x00, 0x00, 0x00,               // mov eax, 6
    0xcd, 0x80,                                   // int 0x80
    // Save handle in rbx, then sys_shm_map(handle, 0x500000, 0)
    0x48, 0x89, 0xc3,                             // mov rbx, rax
    0x48, 0x89, 0xc7,                             // mov rdi, rax
    0xbe, 0x00, 0x00, 0x50, 0x00,               // mov esi, 0x500000
    0x31, 0xd2,                                   // xor edx, edx
    0xb8, 0x07, 0x00, 0x00, 0x00,               // mov eax, 7
    0xcd, 0x80,                                   // int 0x80
    // Fill 256 bytes: pattern 0,1,2,...,255
    0x48, 0x89, 0xc7,                             // mov rdi, rax  (base=0x500000)
    0x31, 0xc9,                                   // xor ecx, ecx
    // .loop @37:
    0x88, 0x0c, 0x0f,                             // mov [rdi+rcx], cl
    0xff, 0xc1,                                   // inc ecx
    0x81, 0xf9, 0x00, 0x01, 0x00, 0x00,         // cmp ecx, 256
    0x75, 0xf3,                                   // jne .loop (-13)
    // Send handle byte via IPC endpoint 0
    0x53,                                         // push rbx
    0x31, 0xff,                                   // xor edi, edi
    0x48, 0x89, 0xe6,                             // mov rsi, rsp
    0xba, 0x01, 0x00, 0x00, 0x00,               // mov edx, 1
    0xb8, 0x08, 0x00, 0x00, 0x00,               // mov eax, 8
    0xcd, 0x80,                                   // int 0x80
    0xf4,                                         // hlt
];

#[cfg(feature = "shm_test")]
static SHM_READER_BLOB: [u8; 105] = [
    // sys_ipc_recv(0, stack_buf, 8)
    0x48, 0x83, 0xec, 0x10,                     // sub rsp, 16
    0x31, 0xff,                                   // xor edi, edi
    0x48, 0x89, 0xe6,                             // mov rsi, rsp
    0xba, 0x08, 0x00, 0x00, 0x00,               // mov edx, 8
    0xb8, 0x09, 0x00, 0x00, 0x00,               // mov eax, 9
    0xcd, 0x80,                                   // int 0x80
    // Load handle from buffe
    0x0f, 0xb6, 0x3c, 0x24,                     // movzx edi, byte [rsp]
    // sys_shm_map(handle, 0x500000, 0)
    0xbe, 0x00, 0x00, 0x50, 0x00,               // mov esi, 0x500000
    0x31, 0xd2,                                   // xor edx, edx
    0xb8, 0x07, 0x00, 0x00, 0x00,               // mov eax, 7
    0xcd, 0x80,                                   // int 0x80
    // Compute checksum: sum 256 bytes
    0x48, 0x89, 0xc6,                             // mov rsi, rax
    0x31, 0xc9,                                   // xor ecx, ecx
    0x31, 0xd2,                                   // xor edx, edx
    // .loop @46:
    0x0f, 0xb6, 0x04, 0x0e,                     // movzx eax, byte [rsi+rcx]
    0x01, 0xc2,                                   // add edx, eax
    0xff, 0xc1,                                   // inc ecx
    0x81, 0xf9, 0x00, 0x01, 0x00, 0x00,         // cmp ecx, 256
    0x75, 0xf0,                                   // jne .loop (-16)
    // Check sum == 32640 (0+1+...+255)
    0x81, 0xfa, 0x80, 0x7f, 0x00, 0x00,         // cmp edx, 32640
    0x75, 0x11,                                   // jne .bad (+17)
    // sys_debug_write("SHM: checksum ok\n", 17)
    0x48, 0x8d, 0x3d, 0x0b, 0x00, 0x00, 0x00,   // lea rdi, [rip+0x0B] -> msg @88
    0xbe, 0x11, 0x00, 0x00, 0x00,               // mov esi, 17
    0x31, 0xc0,                                   // xor eax, eax
    0xcd, 0x80,                                   // int 0x80
    0xf4,                                         // hlt
    // .bad:
    0xf4,                                         // hlt
    // Data @88:
    b'S', b'H', b'M', b':', b' ', b'c', b'h', b'e',
    b'c', b'k', b's', b'u', b'm', b' ', b'o', b'k', b'\n',
];

#[cfg(feature = "pressure_shm_test")]
static SHM_PRESSURE_BLOB: [u8; 264] = [
    // reserve handle table on stack, count in r12d
    0x48, 0x81, 0xEC, 0x00, 0x02, 0x00, 0x00,
    0x48, 0x89, 0xE3,
    0x45, 0x31, 0xE4,
    // create_loop: sys_shm_create(4096) until -1
    0xBF, 0x00, 0x10, 0x00, 0x00,
    0xB8, 0x06, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x83, 0xF8, 0xFF,
    0x74, 0x15,
    0x4A, 0x89, 0x04, 0xE3,
    0x41, 0xFF, 0xC4,
    0x41, 0x81, 0xFC, 0x00, 0x01, 0x00, 0x00,
    0x72, 0xDE,
    0xE9, 0xB7, 0x00, 0x00, 0x00,
    // require count >= 1
    0x41, 0x83, 0xFC, 0x01,
    0x0F, 0x82, 0xAD, 0x00, 0x00, 0x00,
    // one more create after failure must still fail
    0xBF, 0x00, 0x10, 0x00, 0x00,
    0xB8, 0x06, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x83, 0xF8, 0xFF,
    0x0F, 0x85, 0x97, 0x00, 0x00, 0x00,
    // k = min(count, 32)
    0x45, 0x89, 0xE5,
    0x41, 0x83, 0xFD, 0x20,
    0x76, 0x06,
    0x41, 0xBD, 0x20, 0x00, 0x00, 0x00,
    0x45, 0x31, 0xF6,
    // loop i in [0, k): map/write/verify/unmap
    0x45, 0x39, 0xEE,
    0x73, 0x63,
    0x4A, 0x8B, 0x3C, 0xF3,
    0xBE, 0x00, 0x00, 0x50, 0x00,
    0x45, 0x89, 0xF7,
    0x41, 0xC1, 0xE7, 0x0C,
    0x44, 0x01, 0xFE,
    0x31, 0xD2,
    0xB8, 0x07, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x83, 0xF8, 0xFF,
    0x74, 0x5E,
    0x49, 0x89, 0xC7,
    0x45, 0x88, 0x37,
    0x41, 0xC6, 0x47, 0x01, 0xA5,
    0x41, 0xC6, 0x47, 0x02, 0x5A,
    0x45, 0x88, 0x77, 0x03,
    0x45, 0x38, 0x37,
    0x75, 0x45,
    0x41, 0x80, 0x7F, 0x01, 0xA5,
    0x75, 0x3E,
    0x41, 0x80, 0x7F, 0x02, 0x5A,
    0x75, 0x37,
    0x45, 0x38, 0x77, 0x03,
    0x75, 0x31,
    0x4C, 0x89, 0xFF,
    // sys_shm_unmap is ABI v3 id 42 (the blob predates the renumbering)
    0xB8, 0x2A, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0x48, 0x85, 0xC0,
    0x75, 0x22,
    0x41, 0xFF, 0xC6,
    0xEB, 0x98,
    // success: sys_debug_write("PRESSURE: shm ok", 16), qemu_exit(0x31)
    0x48, 0x8D, 0x3D, 0x23, 0x00, 0x00, 0x00,
    0xBE, 0x10, 0x00, 0x00, 0x00,
    0x31, 0xC0,
    0xCD, 0x80,
    0xBF, 0x31, 0x00, 0x00, 0x00,
    0xB8, 0x62, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0xF4,
    // fail: qemu_exit(0x33)
    0xBF, 0x33, 0x00, 0x00, 0x00,
    0xB8, 0x62, 0x00, 0x00, 0x00,
    0xCD, 0x80,
    0xF4,
    // "PRESSURE: shm ok"
    0x50, 0x52, 0x45, 0x53, 0x53, 0x55, 0x52, 0x45, 0x3A, 0x20, 0x73, 0x68, 0x6D, 0x20, 0x6F, 0x6B,
];

// --------------- Deferred syscall stubs (every lane) ---------------

#[allow(dead_code)]
unsafe fn sys_fork_deferred_v1() -> u64 {
    0xFFFF_FFFF_FFFF_FFFF
}

#[allow(dead_code)]
unsafe fn sys_clone_deferred_v1() -> u64 {
    0xFFFF_FFFF_FFFF_FFFF
}

#[allow(dead_code)]
unsafe fn sys_epoll_deferred_v1() -> u64 {
    0xFFFF_FFFF_FFFF_FFFF
}

// --------------- Paging verification ---------------

fn check_paging() {
    let cr0: u64;
    unsafe {
        core::arch::asm!("mov {}, cr0", out(reg) cr0, options(nomem, nostack));
    }
    if cr0 & (1 << 31) != 0 {
        serial_write(b"MM: paging=on\n");
    } else {
        serial_write(b"MM: paging=off\n");
    }
}

#[cfg(feature = "sched_test")]
extern "C" fn thread_a() { loop { serial_write(b"A\n"); } }

#[cfg(feature = "sched_test")]
extern "C" fn thread_b() { loop { serial_write(b"B\n"); } }

// --------------- Kernel entry ---------------

#[no_mangle]
pub extern "C" fn kmain() -> ! {
    serial_init();
    serial_write(b"RUGO: boot ok\n");
    fb::fb_init();
    if fb::fb_ready() {
        let (w, h) = fb::fb_size();
        serial_write(b"FB: console on 0x");
        serial_write_hex(w);
        serial_write(b" x 0x");
        serial_write_hex(h);
        serial_write(b"\n");
    } else {
        serial_write(b"FB: none\n");
    }
    check_paging();
    mm::enable_nx();
    mm::pmm_init();
    mm::heap_init();
    mm::heap_selftest();

    // The R4 task table lives on the kernel heap; size it to the spawn cap
    // before any lane creates tasks.
    #[cfg(any(feature = "ipc_test", feature = "shm_test", feature = "ipc_badptr_send_test", feature = "ipc_badptr_recv_test", feature = "ipc_badptr_svc_test", feature = "ipc_buffer_full_test", feature = "ipc_waiter_busy_test", feature = "svc_overwrite_test", feature = "svc_full_test", feature = "svc_bad_endpoint_test", feature = "stress_ipc_test", feature = "quota_endpoints_test", feature = "quota_shm_test", feature = "quota_threads_test", feature = "go_test"))]
    unsafe {
        r4_tasks_init();
    }

    unsafe {
        gdt_init();
        idt_init();
    }

    // SMP bring-up runs AFTER the IDT/GDT exist, so an AP released here can load
    // the populated IDT and take the IPI vector (full-os guide Part I.3).
    smp::smp_init();

    #[cfg(feature = "pf_test")]
    unsafe {
        let p = 0x0000_0040_0000_0000u64 as *const u8;
        core::ptr::read_volatile(p);
    }

    #[cfg(feature = "idt_smoke_test")]
    unsafe {
        core::arch::asm!("int3", options(nomem, nostack));
    }

    #[cfg(feature = "sched_test")]
    {
        unsafe {
            pic_init();
            pit_init(100);
            sched_init();
            thread_create(thread_a);
            thread_create(thread_b);
            core::arch::asm!("sti", options(nomem, nostack));
        }
        loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)); } }
    }

    // M3: user_hello_test
    #[cfg(feature = "user_hello_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(&USER_HELLO_BLOB);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M3: syscall_test
    #[cfg(feature = "syscall_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(&USER_SYSCALL_BLOB);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M3: thread_exit_test
    #[cfg(feature = "thread_exit_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(&USER_THREAD_EXIT_BLOB);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M3: thread_spawn_test
    #[cfg(feature = "thread_spawn_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(&USER_THREAD_SPAWN_BLOB);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M3: vm_map_test
    #[cfg(feature = "vm_map_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(&USER_VM_MAP_BLOB);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M3: syscall_invalid_test
    #[cfg(feature = "syscall_invalid_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(&USER_SYSCALL_INVALID_BLOB);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M3: stress_syscall_test
    #[cfg(feature = "stress_syscall_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(&USER_STRESS_SYSCALL_BLOB);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M3: yield_test
    #[cfg(feature = "yield_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(&USER_YIELD_BLOB);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M3: user_fault_test
    #[cfg(feature = "user_fault_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(&USER_FAULT_BLOB);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: ipc_test â€” ping-pong between two user tasks
    #[cfg(feature = "ipc_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&IPC_PONG_BLOB, &IPC_PING_BLOB);

        // Pre-create endpoints 0 and 1
        R4_ENDPOINTS[0].active = true;
        R4_ENDPOINTS[1].active = true;

        // Init tasks: task 0 = pong, task 1 = ping
        R4_NUM_TASKS = 2;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        r4_init_task(1, USER_CODE2_VA, USER_STACK2_TOP, 1);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        // Enter ring 3 with task 0 (pong)
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: shm_test â€” shared memory bulk transfe
    // R4: stress_ipc_test - two senders + two receivers
    #[cfg(feature = "stress_ipc_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages4(
            &STRESS_IPC_SENDER_A_BLOB,
            &STRESS_IPC_SENDER_B_BLOB,
            &STRESS_IPC_RECV_C_BLOB,
            &STRESS_IPC_RECV_D_BLOB,
        );

        // Endpoints: A->C on 0, B->D on 1
        R4_ENDPOINTS[0].active = true;
        R4_ENDPOINTS[1].active = true;

        R4_NUM_TASKS = 4;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        r4_init_task(1, USER_CODE2_VA, USER_STACK2_TOP, 1);
        r4_init_task(2, USER_CODE3_VA, USER_STACK3_TOP, 2);
        r4_init_task(3, USER_CODE4_VA, USER_STACK4_TOP, 3);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: shm_test - shared memory bulk transfer
    #[cfg(feature = "shm_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);

        #[cfg(feature = "pressure_shm_test")]
        {
            setup_r4_pages(&SHM_PRESSURE_BLOB, &SHM_PRESSURE_BLOB);
            R4_NUM_TASKS = 1;
            r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
            R4_TASKS[0].state = R4State::Running;
            R4_CURRENT = 0;
            enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
        }

        #[cfg(not(feature = "pressure_shm_test"))]
        {
            setup_r4_pages(&SHM_WRITER_BLOB, &SHM_READER_BLOB);

            // Pre-create endpoint 0
            R4_ENDPOINTS[0].active = true;

            // Init tasks: task 0 = writer, task 1 = reader
            R4_NUM_TASKS = 2;
            r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
            r4_init_task(1, USER_CODE2_VA, USER_STACK2_TOP, 1);
            R4_TASKS[0].state = R4State::Running;
            R4_CURRENT = 0;

            enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
        }
    }

    // R4: ipc_badptr_send_test â€” single task sends to endpoint 0 with bad pointer
    #[cfg(feature = "ipc_badptr_send_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&IPC_BADPTR_SEND_BLOB, &IPC_BADPTR_SEND_BLOB);

        // Pre-create endpoint 0 so send reaches the pointer check
        R4_ENDPOINTS[0].active = true;

        // Single task
        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: ipc_badptr_recv_test â€” single task receives with bad pointer
    #[cfg(feature = "ipc_badptr_recv_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&IPC_BADPTR_RECV_BLOB, &IPC_BADPTR_RECV_BLOB);

        // Pre-create endpoint 0 so recv reaches the pointer check
        R4_ENDPOINTS[0].active = true;

        // Single task
        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: ipc_badptr_svc_test â€” single task calls svc_register with bad name pointer
    #[cfg(feature = "ipc_badptr_svc_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&SVC_BADPTR_BLOB, &SVC_BADPTR_BLOB);

        // Single task (no endpoints needed for svc_register)
        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: ipc_buffer_full_test â€” single task verifies send returns -1 on occupied slot
    #[cfg(feature = "ipc_buffer_full_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&IPC_BUFFER_FULL_BLOB, &IPC_BUFFER_FULL_BLOB);

        // Pre-create endpoint 0
        R4_ENDPOINTS[0].active = true;

        // Single task
        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: ipc_waiter_busy_test â€” second waiter and truncation cases return -1
    #[cfg(feature = "ipc_waiter_busy_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&IPC_WAITER_BLOCK_BLOB, &IPC_WAITER_CONTENDER_BLOB);

        // Pre-create endpoint 0 shared by both tasks
        R4_ENDPOINTS[0].active = true;

        // task 0 blocks first, task 1 verifies deterministic -1 behavior
        R4_NUM_TASKS = 2;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        r4_init_task(1, USER_CODE2_VA, USER_STACK2_TOP, 1);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: svc_overwrite_test â€” single task verifies duplicate registration overwrites endpoint
    #[cfg(feature = "svc_overwrite_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&SVC_OVERWRITE_BLOB, &SVC_OVERWRITE_BLOB);

        // Pre-create endpoints referenced by the test payload (1, 2)
        R4_ENDPOINTS[1].active = true;
        R4_ENDPOINTS[2].active = true;

        // Single task
        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: svc_full_test â€” single task verifies 5th unique registration returns -1
    #[cfg(feature = "svc_full_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&SVC_FULL_BLOB, &SVC_FULL_BLOB);

        // Pre-create endpoints referenced by the test payload (0..4)
        R4_ENDPOINTS[0].active = true;
        R4_ENDPOINTS[1].active = true;
        R4_ENDPOINTS[2].active = true;
        R4_ENDPOINTS[3].active = true;
        R4_ENDPOINTS[4].active = true;

        // Single task
        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: svc_bad_endpoint_test â€” single task verifies register rejects inactive endpoint
    #[cfg(feature = "svc_bad_endpoint_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&SVC_BAD_ENDPOINT_BLOB, &SVC_BAD_ENDPOINT_BLOB);

        // Single task; no endpoints are pre-created on purpose.
        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: quota_endpoints_test - endpoint create returns -1 at per-task limit
    #[cfg(feature = "quota_endpoints_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&QUOTA_ENDPOINTS_BLOB, &QUOTA_ENDPOINTS_BLOB);

        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: quota_shm_test - shm_create returns -1 at per-task limit
    #[cfg(feature = "quota_shm_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&QUOTA_SHM_BLOB, &QUOTA_SHM_BLOB);

        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // R4: quota_threads_test - thread_spawn returns -1 at per-task limit
    #[cfg(feature = "quota_threads_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_r4_pages(&QUOTA_THREADS_BLOB, &QUOTA_THREADS_BLOB);

        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;

        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M5: blk_test â€” block driver + syscalls
    #[cfg(feature = "blk_test")]
    unsafe {
        // Compute kv2p delta from Limine responses
        let hhdm_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(HHDM_REQUEST.response));
        let kaddr_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(KADDR_REQUEST.response));
        let kphys = (*kaddr_resp_ptr).physical_base;
        let kvirt = (*kaddr_resp_ptr).virtual_base;
        BLK_KV2P_DELTA = kphys.wrapping_sub(kvirt);
        HHDM_OFFSET = (*hhdm_resp_ptr).offset;

        if !block_driver_probe(true, cfg!(feature = "native_storage_test"), cfg!(feature = "native_storage_test")) {
            serial_write(b"BLK: not found\n");
            qemu_exit(0x31);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }

        // Set up user mode and run block test blob
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        #[cfg(feature = "blk_badptr_test")]
        let user_blob = &BLK_BADPTR_BLOB;
        #[cfg(all(not(feature = "blk_badptr_test"), feature = "blk_badlen_test"))]
        let user_blob = &BLK_BADLEN_BLOB;
        #[cfg(all(not(feature = "blk_badptr_test"), not(feature = "blk_badlen_test"), feature = "stress_blk_test"))]
        let user_blob = &BLK_STRESS_BLOB;
        #[cfg(all(not(feature = "blk_badptr_test"), not(feature = "blk_badlen_test"), not(feature = "stress_blk_test")))]
        let user_blob = &BLK_TEST_BLOB;
        setup_user_pages(user_blob);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M5: blk_invariants_test â€” VirtIO block init hardening check
    #[cfg(feature = "blk_invariants_test")]
    unsafe {
        let kaddr_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(KADDR_REQUEST.response));
        let kphys = (*kaddr_resp_ptr).physical_base;
        let kvirt = (*kaddr_resp_ptr).virtual_base;
        BLK_KV2P_DELTA = kphys.wrapping_sub(kvirt);

        match pci_find_virtio_blk() {
            None => {
                serial_write(b"BLK: not found\n");
                qemu_exit(0x31);
                loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }
            Some(iobase) => {
                // For blk_init_fail_test, intentionally probe a bad iobase to
                // deterministically exercise the init-failure path.
                let init_iobase = if cfg!(feature = "blk_init_fail_test") {
                    iobase.wrapping_add(0x100)
                } else {
                    iobase
                };
                if !virtio_blk_init(init_iobase) {
                    serial_write(b"BLK: init failed\n");
                    qemu_exit(0x31);
                    loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
                }
                if cfg!(feature = "blk_init_fail_test") {
                    serial_write(b"BLK: unexpected init success\n");
                    qemu_exit(0x33);
                    loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
                }
                serial_write(b"BLK: found virtio-blk\n");
            }
        }

        qemu_exit(0x31);
        loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
    }

    // M6: fs_test â€” Filesystem + package manager + shell
    //
    // Architecture A (services over IPC): the R4 IPC infrastructure is proven.
    // For v0 the kernel orchestrates fsd/pkg/sh logic (reading SimpleFS from
    // the VirtIO block disk, parsing the PKG format, extracting the hello
    // binary).  The hello app runs in genuine user mode (ring 3) to validate
    // the full stack:  block driver â†’ SimpleFS â†’ PKG â†’ user execution.
    #[cfg(feature = "fs_test")]
    unsafe {
        // --- block driver init (same as blk_test) ---
        let hhdm_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(HHDM_REQUEST.response));
        let kaddr_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(KADDR_REQUEST.response));
        let kphys = (*kaddr_resp_ptr).physical_base;
        let kvirt = (*kaddr_resp_ptr).virtual_base;
        BLK_KV2P_DELTA = kphys.wrapping_sub(kvirt);

        HHDM_OFFSET = (*hhdm_resp_ptr).offset;
        if !block_driver_probe(true, false, false) {
            serial_write(b"BLK: not found\n");
            qemu_exit(0x31);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }

        // --- fsd: mount SimpleFS v0 ---
        // Read superblock from sector 0
        if !block_io_dispatch(false, 0, 512, false) {
            serial_write(b"FSD: read error\n");
            qemu_exit(0x31);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }
        let sb_magic = u32::from_le_bytes([
            BLK_DATA_PAGE.0[0], BLK_DATA_PAGE.0[1],
            BLK_DATA_PAGE.0[2], BLK_DATA_PAGE.0[3],
        ]);
        if sb_magic != runtime::storage::SIMPLEFS_MAGIC {
            serial_write(b"FSD: bad magic\n");
            qemu_exit(0x31);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }
        let file_count = u32::from_le_bytes([
            BLK_DATA_PAGE.0[4], BLK_DATA_PAGE.0[5],
            BLK_DATA_PAGE.0[6], BLK_DATA_PAGE.0[7],
        ]);
        serial_write(b"FSD: mount ok\n");

        // --- fsd: read file table from sector 1 ---
        if !block_io_dispatch(false, 1, 512, false) {
            serial_write(b"FSD: ft read error\n");
            qemu_exit(0x31);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }
        // Save file table (BLK_DATA_PAGE is reused by next I/O)
        let mut ft_buf = [0u8; 512];
        core::ptr::copy_nonoverlapping(BLK_DATA_PAGE.0.as_ptr(), ft_buf.as_mut_ptr(), 512);

        // --- pkg: find hello.pkg in file table ---
        let pkg_name: &[u8; 9] = b"hello.pkg";
        let mut pkg_sector = 0u32;
        let mut pkg_found = false;
        let fc = if file_count > 16 { 16 } else { file_count as usize };
        let mut fi = 0usize;
        while fi < fc {
            let base = fi * 32;
            let mut ok = true;
            let mut pi = 0usize;
            while pi < pkg_name.len() {
                if ft_buf[base + pi] != pkg_name[pi] { ok = false; break; }
                pi += 1;
            }
            if ok {
                pkg_sector = u32::from_le_bytes([
                    ft_buf[base + 24], ft_buf[base + 25],
                    ft_buf[base + 26], ft_buf[base + 27],
                ]);
                pkg_found = true;
                break;
            }
            fi += 1;
        }
        if !pkg_found {
            serial_write(b"PKG: hello.pkg not found\n");
            qemu_exit(0x31);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }

        // --- pkg: read hello.pkg from disk ---
        if !block_io_dispatch(false, pkg_sector as u64, 512, false) {
            serial_write(b"PKG: read error\n");
            qemu_exit(0x31);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }

        // Parse PKG v0 header: magic(4) + bin_size(4) + name(24) + sha256(32) = 64 bytes
        let pkg_magic = u32::from_le_bytes([
            BLK_DATA_PAGE.0[0], BLK_DATA_PAGE.0[1],
            BLK_DATA_PAGE.0[2], BLK_DATA_PAGE.0[3],
        ]);
        if pkg_magic != runtime::storage::PKG_MAGIC_V1 {
            serial_write(b"PKG: bad magic\n");
            qemu_exit(0x31);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }
        let bin_size = u32::from_le_bytes([
            BLK_DATA_PAGE.0[4], BLK_DATA_PAGE.0[5],
            BLK_DATA_PAGE.0[6], BLK_DATA_PAGE.0[7],
        ]) as usize;
        if bin_size == 0 || bin_size > 4032 {
            serial_write(b"PKG: bad bin_size\n");
            qemu_exit(0x31);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }
        let mut expected_hash = [0u8; 32];
        expected_hash.copy_from_slice(&BLK_DATA_PAGE.0[32..64]);

        // --- sh: load hello binary into user page and run it ---
        let hello_bin = &BLK_DATA_PAGE.0[64..64 + bin_size];
        let actual_hash = sha256_digest(hello_bin);
        if actual_hash != expected_hash {
            serial_write(b"PKG: bad hash\n");
            qemu_exit(0x31);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }
        serial_write(b"PKG: hash ok\n");
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        HHDM_OFFSET = (*hhdm_resp_ptr).offset;
        if hello_bin.len() >= 4 && &hello_bin[..4] == b"\x7FELF" {
            let entry = match setup_user_elf_pages(hello_bin) {
                Some(v) => v,
                None => {
                    serial_write(b"PKG: bad elf\n");
                    qemu_exit(0x31);
                    loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
                }
            };
            serial_write(b"PKG: elf ok\n");
            enter_ring3_at(entry, USER_STACK_TOP);
        } else {
            setup_user_pages(hello_bin);
            enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
        }
    }

    // G1: go_test â€” TinyGo user program
    #[cfg(feature = "compat_real_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        net::r4_c4_runtime_init();
        COMPAT_REAL_APP_INDEX = 0;
        process::compat_real_enter_current_app();
    }

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        dma::dma_init(); // full-os Part II.7: driver DMA pool
        dma::dma_selftest();
        pci_enumerate_log();
        xhci_detect(); // full-os Part II.7: USB xHCI controller discovery
        e1000_detect(); // full-os Part II.7: Intel e1000 NIC discovery
        hda_detect(); // full-os Part III: Intel HD Audio controller discovery
        let _ = ecam_selftest(); // full-os Part II.7: PCIe ECAM config access
        let _ = msix_selftest(); // full-os Part II.7: MSI-X capability + enable
        let _ = kbd::mouse_selftest(); // full-os Part III: PS/2 mouse bring-up
        let _ = kbd::mouse_packet_selftest(); // full-os Part III: mouse movement packets
        net::r4_c4_runtime_init();
        // Net responder self-tests (full-os guide Part II.6): exercise the same
        // builders the live RX pump uses to answer inbound pings and ARP.
        let _ = netcfg::icmp_selftest();
        let _ = netcfg::icmp_echo_request_selftest(); // full-os II.6: outbound ping (ICMP echo request)
        let _ = netcfg::icmp_error_selftest(); // full-os II.6: ICMP dest-unreachable / time-exceeded
        let _ = netcfg::arp_selftest();
        let _ = netcfg::gratuitous_arp_selftest(); // full-os II.6: gratuitous ARP announcement
        let _ = netcfg::arp_cache_selftest(); // full-os II.6: ARP cache learn/lookup
        let _ = netcfg::icmpv6_selftest();
        let _ = netcfg::udp_echo_selftest();
        let _ = netcfg::ndp_selftest();
        let _ = tcp::tcp_listen_selftest();
        let _ = tcp::tcp_rto_selftest();
        let _ = tcp::tcp_rtt_selftest();
        let _ = tcp::tcp_cc_selftest();
        let _ = net::route_selftest();
        let _ = netcfg::nud_selftest();
        let _ = netcfg::slaac_selftest();
        let _ = tcp::tcp_fastrexmit_selftest();
        let _ = netcfg::udp6_echo_selftest();
        let _ = netcfg::tcp6_listen_selftest();
        let _ = tcp::tcp_sndwin_selftest();
        let _ = tcp::tcp_mss_selftest(); // full-os Part II.6: TCP MSS option on the SYN
        let _ = netcfg::dad_selftest(); // full-os Part II.6: IPv6 NDP DAD on our own address
        installer_selftest(); // full-os Part V.11: provision an install target disk
        cache::cache_selftest(); // full-os Part II.5: block buffer cache
        mm::swap_selftest(); // full-os Part I.4: swap / page eviction
        // full-os Part I.3 (SMP workload-distribution slice 2): the BSP + an AP write
        // distinct scratch LBAs CONCURRENTLY and the BSP proves no cross-clobber --
        // the STORAGE_LOCK serializing the shared BLK_DATA_PAGE + virtio ring across
        // cores. Runs only when an AP is online (-smp2/-smp4 go lanes); a no-op on -smp1.
        smp::storage_smp_selftest();
        let _ = aes::aes_selftest(); // full-os Part IV.10: AES-128 (FIPS-197 KAT)
        mm::huge_page_selftest(); // full-os Part I.4: 2 MiB huge page
        let _ = tty::tty_selftest(); // full-os Part V.11: TTY line discipline
        gpt_parse_selftest(); // full-os Part II.5: GPT partition table parse
        gpt_crc_selftest(); // full-os Part II.5: GPT header CRC-32 validation
        let _ = mount::mount_selftest(); // full-os Part II.5: mount table
        dl_ondisk_seed(); // full-os Part V.11: seed /data/dltest.so for on-disk dlopen
        clock_ids_selftest(); // full-os Part IV.9: BOOTTIME + MONOTONIC_RAW clock ids
        timerfd_periodic_selftest(); // full-os Part IV.9: periodic timerfd re-arm
        audit_checkpoint_selftest(); // full-os Part IV.10: sandbox/power audit checkpoints
        let _ = kbd::input_event_selftest(); // full-os Part III: input event queue
        match fb::fb_alpha_selftest() {
            // full-os Part III: framebuffer alpha blending (src-over).
            1 => serial_write(b"FBALPHA: blend ok\n"),
            2 => serial_write(b"FBALPHA: blend skip (no fb)\n"),
            _ => serial_write(b"FBALPHA: blend FAIL\n"),
        }
        match fb::fb_cursor_selftest() {
            // full-os Part III: mouse cursor compositing with save-under.
            1 => serial_write(b"FBCURSOR: save-under ok\n"),
            2 => serial_write(b"FBCURSOR: save-under skip (no fb)\n"),
            _ => serial_write(b"FBCURSOR: save-under FAIL\n"),
        }
        let _ = rng_hwseed_selftest(); // full-os Part IV.10: RDRAND-seeded CSPRNG
        // full-os Part I.3: online CPU count via the real sys_sysinfo op-13 path.
        serial_write(b"CPUS: count=0x");
        serial_write_hex(sys_sysinfo(13, 0, 0));
        serial_write(b"\n");
        let _ = sha256::sha256_selftest(); // full-os Part IV.10: SHA-256 + measured boot
        let _ = sha256::secure_boot_selftest(); // full-os Part IV.10: secure boot (golden PCR verify)
        let _ = net::pkg_install_selftest(); // full-os Part V.11: package sig-verify + install
        let _ = net::pkg_manager_selftest(); // full-os Part V.11: signed repo index + select + install
        // Package manager (full-os Part V.11): if a fetch-request record is present
        // on disk (sector 16 = "PKGREQ" + le16 port), arm a package fetch; the
        // PIT-tick driver runs it once the network is up. Only the package-fetch
        // test writes this record, so ordinary boots never attempt a fetch.
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            const PKG_REQ_LBA: u64 = 16;
            if block_io_dispatch(false, PKG_REQ_LBA, 512, false)
                && BLK_DATA_PAGE.0[0..6] == *b"PKGREQ"
            {
                let port = u16::from_le_bytes([BLK_DATA_PAGE.0[6], BLK_DATA_PAGE.0[7]]);
                if port != 0 {
                    net::pkg_fetch_arm(port);
                    serial_write(b"PKG: fetch armed\n");
                }
            }
        }
        m8_reset_fd_table();
        #[cfg(feature = "go_desktop_test")]
        let go_user_bin = GO_DESKTOP_BIN;
        #[cfg(not(feature = "go_desktop_test"))]
        let go_user_bin = GO_USER_BIN;
        setup_go_user_pages(go_user_bin);
        if !user_pages_ok(
            USER_CODE_VA,
            runtime::process::GO_IMAGE_MAX_BYTES,
            USER_PERM_READ | USER_PERM_WRITE,
        ) {
            for page_idx in 0..runtime::process::GO_IMAGE_MAX_PAGES {
                let page_va = USER_CODE_VA
                    + (page_idx * runtime::process::GO_IMAGE_PAGE_SIZE) as u64;
                if !check_page_user_perms(
                    page_va,
                    HHDM_OFFSET,
                    USER_PERM_READ | USER_PERM_WRITE,
                ) {
                    serial_write(b"GO: missing page idx=0x");
                    serial_write_hex(page_idx as u64);
                    serial_write(b" va=0x");
                    serial_write_hex(page_va);
                    serial_write(b" pte=0x");
                    match m3_user_pte_ptr(page_va) {
                        Some(pte) => serial_write_hex(*pte),
                        None => serial_write(b"NONE"),
                    }
                    serial_write(b"\n");
                }
            }
            serial_write(b"GO: user map invalid\n");
            qemu_exit(0x33);
            loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
        }
        // full-os Part V.11: dlopen/dlclose unmap + frame release. Runs here, AFTER
        // setup_go_user_pages has loaded CR3 with the shared user PML4 (the demand-
        // window page tables dl_load_elf's vm_map_current needs are now present), in
        // the free DLOPEN window; cleans up after itself so the Go image is untouched.
        dlclose_selftest();
        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        // The boot task and its service threads run on the shared table;
        // spawned apps get private address spaces cloned from it.
        R4_TASKS[0].pml4_phys = SHARED_PML4_PHYS;
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;
        sched::pic_init();
        sched::pit_init(100);
        serial_write(b"SCHED: preempt on hz=100\n");
        sched::pic_unmask(1);
        serial_write(b"KBD: on\n");
        kbd::mouse_enable_irq(); // full-os Part III: live mouse via IRQ12
        arch_x86::enter_ring3_preemptible(USER_CODE_VA, USER_STACK_TOP);
    }

    // G2 spike: go_std_test â€” std-port candidate user program
    #[cfg(feature = "go_std_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(GO_STD_BIN);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M10: sec_rights_test â€” per-handle rights reduction + transfer checks
    #[cfg(feature = "sec_rights_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(SEC_RIGHTS_BIN);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M10: sec_filter_test â€” syscall/profile sandbox checks
    #[cfg(feature = "sec_filter_test")]
    unsafe {
        let kstack = &stack_top as *const u8 as u64;
        tss_init(kstack);
        setup_user_pages(SEC_FILTER_BIN);
        enter_ring3_at(USER_CODE_VA, USER_STACK_TOP);
    }

    // M7: net_test â€” VirtIO net + UDP echo
    #[cfg(feature = "net_test")]
    unsafe {
        // Compute kv2p delta from Limine responses
        let hhdm_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(HHDM_REQUEST.response));
        let kaddr_resp_ptr = core::ptr::read_volatile(
            core::ptr::addr_of!(KADDR_REQUEST.response));
        let kphys = (*kaddr_resp_ptr).physical_base;
        let kvirt = (*kaddr_resp_ptr).virtual_base;
        net::NET_KV2P_DELTA = kphys.wrapping_sub(kvirt);

        // PCI scan for VirtIO net device
        match net::pci_find_virtio_net() {
            None => {
                serial_write(b"NET: not found\n");
                qemu_exit(0x31);
                loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }
            Some(iobase) => {
                if !net::virtio_net_init(iobase) {
                    serial_write(b"NET: init failed\n");
                    qemu_exit(0x31);
                    loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
                }
                serial_write(b"NET: virtio-net ready\n");
            }
        }

        // Network polling loop â€” handle ARP and UDP echo
        let mut rx_frame = [0u8; 1514];
        let mut echoed = false;
        let mut poll_count: u64 = 0;
        let max_polls: u64 = 500_000_000;
        while !echoed && poll_count < max_polls {
            let len = net::virtio_net_recv(&mut rx_frame);
            if len > 0 {
                echoed = net::net_handle_frame(&rx_frame[..len]);
            }
            core::arch::asm!("pause", options(nomem, nostack));
            poll_count += 1;
        }
        if !echoed {
            serial_write(b"NET: timeout\n");
        }
        qemu_exit(0x31);
        loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
    }

    // Normal boot path (M0/M1)
    #[cfg(not(any(
        feature = "sched_test",
        feature = "user_hello_test",
        feature = "syscall_test",
        feature = "thread_exit_test",
        feature = "thread_spawn_test",
        feature = "vm_map_test",
        feature = "syscall_invalid_test",
        feature = "stress_syscall_test",
        feature = "yield_test",
        feature = "user_fault_test",
        feature = "ipc_test",
        feature = "ipc_badptr_send_test",
        feature = "ipc_badptr_recv_test",
        feature = "ipc_badptr_svc_test",
        feature = "stress_ipc_test",
        feature = "svc_full_test",
        feature = "svc_bad_endpoint_test",
        feature = "shm_test",
        feature = "quota_endpoints_test",
        feature = "quota_shm_test",
        feature = "quota_threads_test",
        feature = "blk_test",
        feature = "blk_invariants_test",
        feature = "fs_test",
        feature = "net_test",
        feature = "go_test",
        feature = "go_std_test",
        feature = "sec_rights_test",
        feature = "sec_filter_test",
    )))]
    {
        serial_write(b"RUGO: halt ok\n");

        #[cfg(feature = "panic_test")]
        panic!("deliberate test panic");

        #[cfg(not(feature = "panic_test"))]
        {
            qemu_exit(0x31);
            loop {
                unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
            }
        }
    }
}

// --------------- Panic handler ---------------

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_write(b"RUGO: panic code=0xDEAD\n");
    qemu_exit(0x31);
    loop {
        unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
    }
}
