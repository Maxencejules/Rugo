// SMP bring-up groundwork (gap item 10): the Limine SMP response hands
// us every application processor; each one runs real kernel code (it
// checks in on an atomic counter) and then parks. The scheduler stays
// single-CPU - that is the documented boundary of this slice; the
// counters prove the cores are alive and under kernel control.

#![allow(dead_code)]

use core::sync::atomic::{AtomicU64, Ordering};

use crate::{serial_write, serial_write_hex};

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
    APS_ONLINE.fetch_add(1, Ordering::SeqCst);
    loop {
        unsafe {
            core::arch::asm!("cli; hlt", options(nomem, nostack));
        }
    }
}

/// Start every AP and report. Called once from kmain on the BSP.
pub fn smp_init() {
    unsafe {
        let resp = core::ptr::read_volatile(core::ptr::addr_of!(SMP_REQUEST.response));
        if resp.is_null() {
            serial_write(b"SMP: cpus=0x0000000000000001\n");
            serial_write(b"SMP: aps online=0x0000000000000000\n");
            return;
        }
        let count = (*resp).cpu_count;
        serial_write(b"SMP: cpus=0x");
        serial_write_hex(count);
        serial_write(b"\n");
        let bsp = (*resp).bsp_lapic_id;
        let mut i = 0u64;
        while i < count {
            let info = *(*resp).cpus.add(i as usize);
            if (*info).lapic_id != bsp {
                // The write to goto_address releases the AP.
                core::ptr::write_volatile(
                    core::ptr::addr_of_mut!((*info).goto_address),
                    ap_entry as *const () as u64,
                );
            }
            i += 1;
        }
        // Wait for check-ins (bounded; ~enough for QEMU at any load).
        let expected = count.saturating_sub(1);
        let mut spins = 0u64;
        while APS_ONLINE.load(Ordering::SeqCst) < expected && spins < 200_000_000 {
            core::hint::spin_loop();
            spins += 1;
        }
        serial_write(b"SMP: aps online=0x");
        serial_write_hex(APS_ONLINE.load(Ordering::SeqCst));
        serial_write(b"\n");
    }
}
