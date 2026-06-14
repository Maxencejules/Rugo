// SMP bring-up groundwork (gap item 10): the Limine SMP response hands
// us every application processor; each one runs real kernel code (it
// checks in on an atomic counter) and then parks. The scheduler stays
// single-CPU - that is the documented boundary of this slice; the
// counters prove the cores are alive and under kernel control.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::{serial_write, serial_write_hex};

// SMP kernel locking (full-os guide Part I.3): a test-and-set spinlock guarding
// a deliberately NON-atomic shared counter. Every CPU (BSP + each AP) hammers
// it `SMP_LOCK_ITERS` times before parking; if the lock works the total is
// exactly cpus*ITERS, while a broken lock loses updates under real contention.
static SMP_LOCK: AtomicU32 = AtomicU32::new(0);
static mut SMP_GUARDED: u64 = 0;
const SMP_LOCK_ITERS: u64 = 2000;

unsafe fn smp_lock_acquire() {
    while SMP_LOCK
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}

unsafe fn smp_lock_release() {
    SMP_LOCK.store(0, Ordering::Release);
}

/// One CPU's contribution: ITERS locked, intentionally non-atomic increments of
/// the shared counter. Read-modify-write through volatile so the compiler can't
/// fuse it; correctness depends entirely on the spinlock serializing CPUs.
unsafe fn smp_lock_hammer() {
    let mut i = 0u64;
    while i < SMP_LOCK_ITERS {
        smp_lock_acquire();
        let v = core::ptr::read_volatile(core::ptr::addr_of!(SMP_GUARDED));
        core::ptr::write_volatile(core::ptr::addr_of_mut!(SMP_GUARDED), v + 1);
        smp_lock_release();
        i += 1;
    }
}

#[repr(C)]
struct LimineSmpInfo {
    processor_id: u32,
    lapic_id: u32,
    reserved: u64,
    goto_address: u64,
    extra_argument: u64,
}

#[repr(C)]
struct LimineSmpResponse {
    revision: u64,
    flags: u32,
    bsp_lapic_id: u32,
    cpu_count: u64,
    cpus: *const *mut LimineSmpInfo,
}

#[repr(C)]
struct LimineSmpRequest {
    id: [u64; 4],
    revision: u64,
    response: *const LimineSmpResponse,
    flags: u64,
}

unsafe impl Sync for LimineSmpRequest {}

#[used]
#[link_section = ".limine_requests"]
static mut SMP_REQUEST: LimineSmpRequest = LimineSmpRequest {
    id: [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b,
         0x95a67b819a1b857e, 0xa0b61b723b6a73e0],
    revision: 0,
    response: core::ptr::null(),
    flags: 0,
};

static APS_ONLINE: AtomicU64 = AtomicU64::new(0);

/// AP entry: Limine hands each AP its own stack and the kernel's page
/// tables. Check in and park - no prints (serial is not multi-CPU
/// safe), no shared kernel state beyond the atomic.
extern "C" fn ap_entry(_info: *const LimineSmpInfo) -> ! {
    // Contend on the spinlock BEFORE checking in, so once the BSP sees all
    // APs online every AP's locked increments are already committed.
    unsafe {
        smp_lock_hammer();
    }
    APS_ONLINE.fetch_add(1, Ordering::SeqCst);
    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
}

/// Start every AP and report. Called once from kmain on the BSP. Also runs a
/// spinlock contention self-test: every CPU hammers a lock-guarded counter and
/// the BSP verifies no updates were lost (full-os guide Part I.3 kernel locking).
pub fn smp_init() {
    unsafe {
        let resp = core::ptr::read_volatile(core::ptr::addr_of!(SMP_REQUEST.response));
        let count = if resp.is_null() { 1 } else { (*resp).cpu_count };
        serial_write(b"SMP: cpus=0x");
        serial_write_hex(count);
        serial_write(b"\n");
        if !resp.is_null() {
            let bsp = (*resp).bsp_lapic_id;
            let mut i = 0u64;
            while i < count {
                let info = *(*resp).cpus.add(i as usize);
                if (*info).lapic_id != bsp {
                    // The write to goto_address releases the AP (it begins
                    // hammering the spinlock immediately).
                    core::ptr::write_volatile(
                        core::ptr::addr_of_mut!((*info).goto_address),
                        ap_entry as *const () as u64,
                    );
                }
                i += 1;
            }
        }
        // The BSP joins the contention while the APs run.
        smp_lock_hammer();
        // Wait for check-ins (bounded; ~enough for QEMU at any load). Each AP
        // checks in only after finishing its locked increments.
        let expected = count.saturating_sub(1);
        let mut spins = 0u64;
        while APS_ONLINE.load(Ordering::SeqCst) < expected && spins < 200_000_000 {
            core::hint::spin_loop();
            spins += 1;
        }
        serial_write(b"SMP: aps online=0x");
        serial_write_hex(APS_ONLINE.load(Ordering::SeqCst));
        serial_write(b"\n");
        // Verify the spinlock serialized every CPU: total must be cpus*ITERS.
        let total = core::ptr::read_volatile(core::ptr::addr_of!(SMP_GUARDED));
        let want = count.wrapping_mul(SMP_LOCK_ITERS);
        serial_write(b"SMP: lock count=0x");
        serial_write_hex(total);
        if total == want {
            serial_write(b" ok\n");
        } else {
            serial_write(b" FAIL\n");
        }
    }
}
