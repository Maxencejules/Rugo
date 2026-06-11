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
pub(crate) mod fb;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod kbd;
mod memory;
pub(crate) mod mm;
mod net;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) mod tcp;
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
                let node = match vfs::vfs_open(rel, create) {
                    Some(n) => n,
                    None => {
                        return 0xFFFF_FFFF_FFFF_FFFF;
                    }
                };
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
            #[cfg(not(feature = "go_test"))]
            _ => 0xFFFF_FFFF_FFFF_FFFF,
        }
    }

    /// sys_fs_ctl (ABI v3.x id 47): namespace mutations and stat for the
    /// /data tree. op 1 = mkdir, 2 = unlink, 3 = stat (returns
    /// kind << 32 | size). op 4 = pipe create (returns rfd << 8 | wfd).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn sys_fs_ctl_v1(op: u64, path_ptr: u64, _arg: u64) -> u64 {
        const ERR: u64 = 0xFFFF_FFFF_FFFF_FFFF;
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
                if R4_TASKS[R4_CURRENT].fd_count != 0 {
                    R4_TASKS[R4_CURRENT].fd_count -= 1;
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
        match op {
            1 => {
                if vfs::vfs_mkdir(rel) { 0 } else { ERR }
            }
            2 => {
                if vfs::vfs_unlink(rel) { 0 } else { ERR }
            }
            3 => match vfs::vfs_stat(rel) {
                Some((kind, size)) => ((kind as u64) << 32) | size as u64,
                None => ERR,
            },
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
                R4_TASKS[R4_CURRENT].fd_count,
                R4_TASKS[R4_CURRENT].fd_limit as usize,
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
                    R4_TASKS[R4_CURRENT].fd_count += 1;
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
        // Slot reuse must not leak signal state into the next task.
        R4_TASKS[tid].sig_handler = 0;
        R4_TASKS[tid].sig_pending = 0;
        R4_TASKS[tid].sig_in_handler = false;
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
            if i != exclude && R4_TASKS[i].state == R4State::Ready {
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
        for i in 0..22 { *frame.add(i) = R4_TASKS[tid].saved_frame[i]; }
        R4_TASKS[tid].state = R4State::Running;
        R4_TASKS[tid].dispatch_count += 1;
        R4_CURRENT = tid;
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
                R4_TASKS[R4_CURRENT].sig_handler = a2;
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
                if !R4_TASKS[R4_CURRENT].sig_in_handler {
                    *frame.add(14) = ERR;
                    return;
                }
                R4_TASKS[R4_CURRENT].sig_in_handler = false;
                for i in 0..22 {
                    *frame.add(i) = R4_TASKS[R4_CURRENT].sig_saved_frame[i];
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
        if r4_copy_wait_status(status_ptr, R4_TASKS[child_tid].exit_status) {
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

    /// PIT tick entry for the default lane: EOI first, then involuntarily
    /// switch the interrupted ring-3 task to the next Ready task. Unlike the
    /// yield path this preserves RAX - the saved frame is not a syscall
    /// return.
    #[cfg(feature = "go_test")]
    pub(crate) unsafe fn r4_timer_preempt(frame: *mut u64) {
        R4_PREEMPT_TICKS += 1;
        sched::pic_send_eoi(0);
        #[cfg(not(feature = "compat_real_test"))]
        if tcp::tcp_active() {
            // Drive the wire TCP machine from the tick while a connection
            // is in flight.
            net::net_rx_pump();
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
            // Release the single-occupancy exec app window on any exit
            // path, including fault containment.
            if EXEC_APP_TID == cur as i32 {
                EXEC_APP_TID = -1;
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
                // All tasks done â€” exit
                let kstack = &stack_top as *const u8 as u64;
                *frame.add(17) = r4_all_done as *const () as u64;
                *frame.add(18) = 0x08;
                *frame.add(19) = 0x02;
                *frame.add(20) = kstack;
                *frame.add(21) = 0x10;
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

    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn exec_log(name: &[u8], what: &[u8]) {
        serial_write(b"EXEC: ");
        serial_write(name);
        serial_write(b" ");
        serial_write(what);
        serial_write(b"\n");
    }

    /// Validate and copy an ET_EXEC ELF64 whose segments live entirely in
    /// the exec app window. Returns the entry point. The destination pages
    /// are demand-paged; copyout_user pre-maps them.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    unsafe fn exec_load_app(image: &[u8]) -> Option<u64> {
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
        let zeros = [0u8; 512];
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
                if copyout_user(p_vaddr, &image[p_offset..p_offset + p_filesz], p_filesz)
                    .is_err()
                {
                    return None;
                }
                let mut z = p_filesz;
                while z < p_memsz {
                    let chunk = core::cmp::min(zeros.len(), p_memsz - z);
                    if copyout_user(p_vaddr + z as u64, &zeros[..chunk], chunk).is_err() {
                        return None;
                    }
                    z += chunk;
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
        if R4_TASKS[R4_CURRENT].cap_flags & R4_TASK_CAP_STORAGE == 0 {
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
        if EXEC_APP_TID >= 0 {
            exec_log(name, b"busy");
            return ERR;
        }
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

        if !block_io_dispatch(false, EXEC_APP_REGION_SECTOR + 1, 512, false) {
            exec_log(name, b"ioerr");
            return ERR;
        }
        let mut table = [0u8; 512];
        core::ptr::copy_nonoverlapping(BLK_DATA_PAGE.0.as_ptr(), table.as_mut_ptr(), 512);

        let fc = core::cmp::min(file_count, 16);
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

        let entry = match exec_load_app(elf) {
            Some(e) => e,
            None => {
                exec_log(name, b"badelf");
                return ERR;
            }
        };

        let parent = R4_CURRENT;
        let tid = match r4_find_spawn_slot() {
            Some(t) => t,
            None => {
                exec_log(name, b"full");
                return ERR;
            }
        };
        // Deliver the argument string (NUL terminated) to the args page;
        // copyout pre-maps it.
        if copyout_user(EXEC_ARGS_VA, &args_buf[..args_n + 1], args_n + 1).is_err() {
            exec_log(name, b"badargs");
            return ERR;
        }
        if !spawn_stdio_ok(stdin_fd, M8FdKind::PipeR)
            || !spawn_stdio_ok(stdout_fd, M8FdKind::PipeW)
        {
            exec_log(name, b"badfd");
            return ERR;
        }
        r4_init_task(tid, entry, r4_stack_top_for_slot(tid), parent);
        // External apps get read access to the file tree (storage) but no
        // network, spawn, or IPC surface.
        R4_TASKS[tid].can_spawn = false;
        R4_TASKS[tid].cap_flags = R4_TASK_CAP_STORAGE;
        R4_TASKS[tid].fd_limit = 4;
        R4_TASKS[tid].socket_limit = 0;
        R4_TASKS[tid].endpoint_limit = 0;
        R4_TASKS[tid].isolation_domain = 5;
        // Hand over the pipe ends: ownership moves to the child so its
        // exit (or fault) releases them and EOF propagates.
        for fdv in [stdin_fd, stdout_fd] {
            if fdv != u64::MAX {
                let idx = fdv as usize;
                M8_FD_TABLE[idx].owner_tid = tid;
                if R4_TASKS[R4_CURRENT].fd_count != 0 {
                    R4_TASKS[R4_CURRENT].fd_count -= 1;
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
        exec_log(name, b"ok");
        tid as u64
    }

    unsafe fn r4_find_spawn_slot() -> Option<usize> {
        for tid in 1..R4_NUM_TASKS {
            if R4_TASKS[tid].state == R4State::Dead {
                return Some(tid);
            }
        }
        if R4_NUM_TASKS >= R4_MAX_TASKS {
            return None;
        }
        let tid = R4_NUM_TASKS;
        R4_NUM_TASKS += 1;
        Some(tid)
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
        R4_TASKS[R4_CURRENT].cap_flags & flag != 0
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
                R4_TASKS[R4_CURRENT].thread_count,
                MAX_THREADS_PER_PROC,
            ) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            if !runtime::isolation::under_quota(R4_THREADS_CREATED, MAX_THREADS_GLOBAL) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            let tid = R4_TASKS[R4_CURRENT].thread_count as u64;
            R4_TASKS[R4_CURRENT].thread_count += 1;
            R4_THREADS_CREATED += 1;
            return tid;
        }
        #[cfg(all(feature = "go_test", not(feature = "quota_threads_test")))]
        {
            if entry >= 0x0000_8000_0000_0000 {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            if !R4_TASKS[R4_CURRENT].can_spawn {
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
                R4_TASKS[R4_CURRENT].endpoint_limit as usize
            } else {
                MAX_ENDPOINTS_PER_PROC
            };
            if !runtime::isolation::under_quota(
                R4_TASKS[R4_CURRENT].endpoint_count,
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
                    R4_TASKS[R4_CURRENT].endpoint_count += 1;
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
        R4_TASKS[R4_CURRENT].ipc_send_count += 1;

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
            if copyout_user(R4_TASKS[wt].recv_buf, &kbuf[..n], n).is_err() {
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
            R4_TASKS[R4_CURRENT].ipc_recv_count += 1;
            *frame.add(14) = n as u64;
            return;
        }

        // A second waiter on the same endpoint is rejected explicitly.
        if R4_ENDPOINTS[ep].waiter >= 0 && R4_ENDPOINTS[ep].waiter != R4_CURRENT as i32 {
            *frame.add(14) = 0xFFFF_FFFF_FFFF_FFFF;
            return;
        }

        // No message â€” block current task and switch
        R4_TASKS[R4_CURRENT].recv_ep = endpoint;
        R4_TASKS[R4_CURRENT].recv_buf = buf;
        R4_TASKS[R4_CURRENT].recv_cap = cap_n as u64;
        r4_save_frame(frame, R4_CURRENT);
        R4_TASKS[R4_CURRENT].block_count += 1;
        R4_TASKS[R4_CURRENT].state = R4State::Blocked;
        R4_ENDPOINTS[ep].waiter = R4_CURRENT as i32;

        match r4_find_ready(R4_CURRENT) {
            Some(tid) => { r4_switch_to(frame, tid); }
            None => {
                // Deadlock â€” no ready tasks
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
                R4_TASKS[R4_CURRENT].shm_count,
                MAX_SHM_PER_PROC,
            ) {
                return 0xFFFF_FFFF_FFFF_FFFF;
            }
            for i in 0..R4_MAX_SHM {
                if !R4_SHM_OBJECTS[i].active {
                    R4_SHM_OBJECTS[i].active = true;
                    R4_SHM_OBJECTS[i].size = 4096;
                    core::ptr::write_bytes(R4_SHM_PAGES[i].0.as_mut_ptr(), 0, 4096);
                    R4_TASKS[R4_CURRENT].shm_count += 1;
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

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn block_flush_dispatch() -> bool {
    match ACTIVE_BLOCK_DRIVER {
        ActiveBlockDriver::VirtioLegacy => true,
        ActiveBlockDriver::Nvme => runtime::native::nvme_flush(),
        ActiveBlockDriver::None => false,
    }
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
        net::r4_c4_runtime_init();
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
        R4_NUM_TASKS = 1;
        r4_init_task(0, USER_CODE_VA, USER_STACK_TOP, 0);
        R4_TASKS[0].state = R4State::Running;
        R4_CURRENT = 0;
        sched::pic_init();
        sched::pit_init(100);
        serial_write(b"SCHED: preempt on hz=100\n");
        sched::pic_unmask(1);
        serial_write(b"KBD: on\n");
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
