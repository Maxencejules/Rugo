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

// SMP IPI (full-os guide Part I.3): x2APIC inter-processor interrupt, the
// prerequisite for cross-CPU work (TLB shootdown, per-CPU scheduling). Gated on
// cpu_count > 1 so the default -smp 1 lanes never enable the LAPIC (which could
// perturb PIC-delivered timer interrupts on the BSP).
static IPI_ACK: AtomicU64 = AtomicU64::new(0);
const IPI_VECTOR: u64 = 240;

unsafe fn rdmsr(msr: u32) -> u64 {
    let (lo, hi): (u32, u32);
    core::arch::asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi,
                     options(nomem, nostack));
    ((hi as u64) << 32) | lo as u64
}

unsafe fn wrmsr(msr: u32, val: u64) {
    core::arch::asm!("wrmsr", in("ecx") msr, in("eax") val as u32,
                     in("edx") (val >> 32) as u32, options(nomem, nostack));
}

/// Enable x2APIC mode + software-enable the local APIC. The spurious vector is
/// 65, whose IDT gate is installed unconditionally in `idt_init` (so a spurious
/// delivery always lands on a present gate, in every lane). Requires CPU x2APIC
/// support (the SMP test boots `-cpu qemu64,+x2apic`). Called per-CPU only when
/// SMP is active.
unsafe fn x2apic_enable() {
    const IA32_APIC_BASE: u32 = 0x1B;
    const X2APIC_SVR: u32 = 0x80F;
    let base = rdmsr(IA32_APIC_BASE);
    // The SDM permits only disabled(00)->xAPIC(10)->x2APIC(11); a direct
    // disabled->x2APIC write #GPs. Set the global-enable bit (11) first, then
    // the x2APIC bit (10), so the transition is legal regardless of the state
    // the firmware/bootloader handed us (QEMU+Limine arrive in xAPIC mode, but
    // do not depend on it).
    wrmsr(IA32_APIC_BASE, base | (1 << 11)); // ensure xAPIC global-enable
    wrmsr(IA32_APIC_BASE, base | (1 << 11) | (1 << 10)); // then enable x2APIC
    wrmsr(X2APIC_SVR, (1 << 8) | 65); // APIC software-enable + spurious vector
}

/// End-of-interrupt for the current CPU's local APIC (x2APIC EOI register).
unsafe fn x2apic_eoi() {
    wrmsr(0x80B, 0);
}

/// Send a fixed IPI to all other CPUs (ICR is one 64-bit MSR write in x2APIC).
/// Destination shorthand 11 = "all excluding self" avoids dest-id matching.
unsafe fn x2apic_broadcast_ipi(vector: u64) {
    wrmsr(0x830, (3 << 18) | (1 << 14) | vector); // all-but-self, assert, fixed
}

/// IPI service routine (from trap_handler vector 240): record the delivery and
/// acknowledge. Runs on the AP that received the IPI; touches no BSP state.
pub(crate) unsafe fn ipi_handler() {
    IPI_ACK.fetch_add(1, Ordering::SeqCst);
    x2apic_eoi();
}

// Per-CPU LAPIC timer (full-os guide Part I.3): each AP's own preemption clock,
// the source a per-CPU scheduler needs (the legacy PIC timer only reaches the
// BSP). v1 just counts ticks to prove every AP's local timer fires.
static AP_TICKS: AtomicU64 = AtomicU64::new(0);
const LAPIC_TIMER_VECTOR: u64 = 241;

/// Start this CPU's LAPIC timer in periodic mode on vector 241.
unsafe fn lapic_timer_start() {
    wrmsr(0x83E, 0x3); // divide configuration: divide by 16
    wrmsr(0x832, (1 << 17) | LAPIC_TIMER_VECTOR); // LVT timer: periodic + vector
    wrmsr(0x838, 0x0010_0000); // initial count (counts down, reloads each period)
}

/// LAPIC-timer service routine (from trap_handler vector 241): tick + EOI.
pub(crate) unsafe fn lapic_timer_handler() {
    AP_TICKS.fetch_add(1, Ordering::SeqCst);
    x2apic_eoi();
}

// TLB shootdown (full-os guide Part I.3): cross-CPU TLB invalidation — the
// mechanism the VM (munmap/mprotect/CoW) and a per-CPU scheduler need once a
// page table is edited on one CPU while another may hold a stale translation.
// The initiator (the BSP today) publishes the target address, then broadcasts
// vector 242; every AP invalidates and increments the ack counter.
static SHOOTDOWN_ADDR: AtomicU64 = AtomicU64::new(0);
static SHOOTDOWN_ACK: AtomicU64 = AtomicU64::new(0);
// Number of APs the initiator should wait for (set once the APs are confirmed
// online in smp_init). 0 => uniprocessor: a shootdown is purely local.
static SMP_AP_COUNT: AtomicU64 = AtomicU64::new(0);
const TLB_SHOOTDOWN_VECTOR: u64 = 242;

/// Invalidate `addr` on the current CPU, or reload CR3 (full flush) if `addr`
/// is 0. Shared by the local path in `tlb_shootdown` and the AP handler.
#[inline(always)]
unsafe fn tlb_invalidate(addr: u64) {
    if addr != 0 {
        core::arch::asm!("invlpg [{}]", in(reg) addr, options(nostack));
    } else {
        let cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
    }
}

/// TLB-shootdown service routine (from trap_handler vector 242): invalidate the
/// requested address (or full-flush when 0), then acknowledge. Runs on each AP.
pub(crate) unsafe fn tlb_shootdown_handler() {
    let addr = SHOOTDOWN_ADDR.load(Ordering::Acquire);
    tlb_invalidate(addr);
    SHOOTDOWN_ACK.fetch_add(1, Ordering::SeqCst);
    x2apic_eoi();
}

/// Initiate a TLB shootdown from the BSP: invalidate `addr` locally, then direct
/// every online AP to do the same and wait (bounded) for their acknowledgements.
/// `addr == 0` requests a full flush (CR3 reload) on each CPU. Returns true if
/// every AP acknowledged (trivially true on a uniprocessor / before APs are up).
///
/// v1 boundary: a single shared `SHOOTDOWN_ADDR` + the BSP-only initiator make
/// this safe for the boot self-test and for callers serialized by the kernel's
/// big lock; a fully concurrent kernel needs per-CPU shootdown mailboxes or a
/// lock around the request so two initiators cannot clobber the address.
pub(crate) unsafe fn tlb_shootdown(addr: u64) -> bool {
    tlb_invalidate(addr); // initiator invalidates its own TLB first
    let aps = SMP_AP_COUNT.load(Ordering::Acquire);
    if aps == 0 {
        return true; // uniprocessor (or APs not up): nothing to shoot down
    }
    let base = SHOOTDOWN_ACK.load(Ordering::SeqCst);
    SHOOTDOWN_ADDR.store(addr, Ordering::Release);
    x2apic_broadcast_ipi(TLB_SHOOTDOWN_VECTOR);
    let mut spins = 0u64;
    while SHOOTDOWN_ACK.load(Ordering::SeqCst) < base + aps && spins < 200_000_000 {
        core::hint::spin_loop();
        spins += 1;
    }
    SHOOTDOWN_ACK.load(Ordering::SeqCst) >= base + aps
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
    unsafe {
        // Contend on the spinlock BEFORE checking in, so once the BSP sees all
        // APs online every AP's locked increments are already committed.
        smp_lock_hammer();
        // Become IPI-ready. Limine hands the AP its OWN GDT, so load the kernel
        // GDT first (else CS=0x08 in the IDT gates would be wrong and the first
        // interrupt would triple-fault); then the shared IDT, then this AP's
        // x2APIC; then check in and park with interrupts enabled.
        crate::arch_x86::gdt_init();
        crate::arch_x86::load_idt();
        x2apic_enable();
        lapic_timer_start(); // this AP's own periodic preemption clock
        APS_ONLINE.fetch_add(1, Ordering::SeqCst);
        loop {
            core::arch::asm!("sti; hlt", options(nomem, nostack));
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
        let mut first_ap: u32 = 0;
        let mut have_ap = false;
        if !resp.is_null() {
            let bsp = (*resp).bsp_lapic_id;
            let mut i = 0u64;
            while i < count {
                let info = *(*resp).cpus.add(i as usize);
                if (*info).lapic_id != bsp {
                    if !have_ap {
                        first_ap = (*info).lapic_id;
                        have_ap = true;
                    }
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
        let online = APS_ONLINE.load(Ordering::SeqCst);
        // Record how many APs are alive so tlb_shootdown knows how many acks to
        // await. Only the confirmed-online APs (they will also ack the IPI and
        // run their timers below), so a degraded boot does not wedge a shootdown.
        SMP_AP_COUNT.store(online, Ordering::Release);
        serial_write(b"SMP: aps online=0x");
        serial_write_hex(online);
        serial_write(b"\n");
        // Verify the spinlock serialized every CPU: total must be cpus*ITERS.
        // Read the deliberately non-atomic SMP_GUARDED ONLY once every AP has
        // checked in: each AP does its SeqCst APS_ONLINE increment AFTER its
        // locked increments, so a successful wait establishes happens-before and
        // no AP is still writing the counter. On the bounded-spin timeout path
        // an AP could still be hammering, so this plain read would race it.
        if online >= expected {
            let total = core::ptr::read_volatile(core::ptr::addr_of!(SMP_GUARDED));
            let want = count.wrapping_mul(SMP_LOCK_ITERS);
            serial_write(b"SMP: lock count=0x");
            serial_write_hex(total);
            if total == want {
                serial_write(b" ok\n");
            } else {
                serial_write(b" FAIL\n");
            }
        } else {
            serial_write(b"SMP: lock count timeout FAIL\n");
        }
        // Inter-processor interrupt: only when SMP is actually present, so the
        // default -smp 1 lanes never enable the LAPIC on the BSP.
        if have_ap {
            let _ = first_ap;
            x2apic_enable(); // the BSP must enable its LAPIC to send the ICR
            x2apic_broadcast_ipi(IPI_VECTOR);
            let mut ispins = 0u64;
            while IPI_ACK.load(Ordering::SeqCst) < expected && ispins < 60_000_000 {
                core::hint::spin_loop();
                ispins += 1;
            }
            serial_write(b"SMP: ipi ack=0x");
            serial_write_hex(IPI_ACK.load(Ordering::SeqCst));
            serial_write(b"\n");
            // Let the APs' periodic LAPIC timers tick, then confirm every AP's
            // own preemption clock fired (the basis for per-CPU scheduling).
            let mut tspins = 0u64;
            while AP_TICKS.load(Ordering::SeqCst) < expected && tspins < 200_000_000 {
                core::hint::spin_loop();
                tspins += 1;
            }
            let ticked = AP_TICKS.load(Ordering::SeqCst);
            serial_write(b"SMP: ap timers ");
            if ticked >= expected {
                serial_write(b"ok\n");
            } else {
                serial_write(b"FAIL\n");
            }
            // TLB shootdown: direct every AP to invalidate a specific address and
            // acknowledge — proof the cross-CPU invalidation path (the mechanism
            // munmap/mprotect/CoW and a per-CPU scheduler need) works end to end.
            // The probe is the address of SMP_GUARDED, a known-mapped kernel VA;
            // invlpg on it is harmless (drops the entry if cached, else a no-op),
            // so what is verified is that every AP executed the directed flush.
            let probe = core::ptr::addr_of!(SMP_GUARDED) as u64;
            let shot_ok = tlb_shootdown(probe);
            serial_write(b"SMP: tlb shootdown ");
            if shot_ok {
                serial_write(b"ok\n");
            } else {
                serial_write(b"FAIL\n");
            }
        }
    }
}
