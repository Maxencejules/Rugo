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

/// The bootstrap processor's x2APIC ID, recorded in `smp_init`. Lets any CPU tell
/// whether it is the BSP without reading GS (the BSP's GS base is left unset because
/// ring-3 TinyGo uses GS), so it is safe in the syscall path on any core.
static BSP_LAPIC_ID: AtomicU32 = AtomicU32::new(0xFFFF_FFFF);

/// This CPU's x2APIC ID (IA32_X2APIC_APICID, MSR 0x802). Requires x2APIC enabled.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn current_apic_id() -> u32 {
    (rdmsr(0x802) & 0xFFFF_FFFF) as u32
}

/// True if the calling CPU is the bootstrap processor. On a uniprocessor (or before
/// any AP checked in) x2APIC may be disabled, so reading the ID MSR could #GP — there
/// we are unconditionally the BSP. Otherwise compare the live x2APIC ID to the BSP's.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn is_bsp() -> bool {
    if SMP_AP_COUNT.load(Ordering::Relaxed) == 0 {
        return true;
    }
    current_apic_id() == BSP_LAPIC_ID.load(Ordering::Relaxed)
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

// Per-CPU data via GS base (full-os guide Part I.3): the storage model a per-CPU
// scheduler needs — each CPU's `current` task, run queue, and stats live in a
// slot reached through the GS base, so an interrupt handler can touch THIS CPU's
// data with no locking and no "which CPU am I" lookup. v1 proves the mechanism:
// each AP points IA32_GS_BASE at its own slot, records its index THROUGH GS, and
// its LAPIC-timer ISR bumps a per-CPU counter THROUGH GS. The BSP's GS base is
// left untouched (the go lane runs userspace on the BSP), and only APs take the
// LAPIC-timer vector, so no kernel `gs:` access ever runs without a base set.
const MAX_CPUS: usize = 64;
const IA32_GS_BASE: u32 = 0xC000_0101;

#[derive(Clone, Copy)]
#[repr(C)]
struct PerCpu {
    cpu_index: u64,    // gs:[0] — written by each CPU through its own GS base
    timer_ticks: u64,  // gs:[8] — bumped by the per-CPU LAPIC-timer ISR
    current_task: u64, // gs:[16] — the task this CPU is currently running (0 = idle)
}

static mut PERCPU: [PerCpu; MAX_CPUS] =
    [PerCpu { cpu_index: 0, timer_ticks: 0, current_task: 0 }; MAX_CPUS];
static PERCPU_NEXT: AtomicU64 = AtomicU64::new(1); // slot 0 reserved for the BSP

/// Point this CPU's GS base at PERCPU[slot] and record its index through GS.
/// Writing cpu_index via `gs:[0]` (rather than PERCPU[slot] directly) is the
/// load-bearing proof: if the GS base is wrong the value lands in the wrong slot
/// and the BSP's verification fails.
unsafe fn percpu_init(slot: usize) {
    let base = core::ptr::addr_of!(PERCPU[slot]) as u64;
    wrmsr(IA32_GS_BASE, base);
    core::arch::asm!("mov qword ptr gs:[0], {v}", v = in(reg) slot as u64, options(nostack));
}

// Cross-CPU work dispatch (full-os guide Part I.3): the BSP hands a kernel work
// item to the application processors; exactly one AP CLAIMS it (atomic CAS),
// runs it on its own core, and reports the result. This is the execution
// primitive a per-CPU scheduler builds on — the APs run real dispatched work
// instead of only parking. v1 ships one work kind (a checkable computation) and
// a single in-flight item (the boot self-test); a real run queue is the capstone.
static WORK_GEN: AtomicU64 = AtomicU64::new(0); // 0 = no work; nonzero = a generation
static WORK_CLAIM: AtomicU64 = AtomicU64::new(0); // the gen an AP CASes to 0 to claim
static WORK_KIND: AtomicU64 = AtomicU64::new(0);
static WORK_ARG: AtomicU64 = AtomicU64::new(0);
static WORK_RESULT: AtomicU64 = AtomicU64::new(0);
static WORK_DONE: AtomicU64 = AtomicU64::new(0); // set to the gen when finished

// Cross-CPU PMM contention self-test (full-os Part I.3, kernel-wide locking /
// slice 5). The AP and BSP allocate a batch of frames CONCURRENTLY (released
// together by a barrier) and the BSP verifies the two batches are disjoint -- a
// frame handed to two CPUs would mean the PMM lock failed to serialize the bitmap
// read-modify-write.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const PMM_TEST_N: usize = 256;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut AP_PMM_FRAMES: [u64; PMM_TEST_N] = [0; PMM_TEST_N];
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static PMM_TEST_AP_READY: AtomicU64 = AtomicU64::new(0); // AP reached the barrier
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static PMM_TEST_GO: AtomicU64 = AtomicU64::new(0); // BSP releases both to allocate

/// AP side of the PMM contention test (work kind 3): announce arrival at the
/// barrier, wait for the BSP's go, then allocate PMM_TEST_N frames into the shared
/// array as fast as possible -- concurrently with the BSP doing the same.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn pmm_test_ap_alloc() {
    PMM_TEST_AP_READY.store(1, Ordering::Release);
    let mut s = 0u64;
    while PMM_TEST_GO.load(Ordering::Acquire) == 0 && s < 500_000_000 {
        core::hint::spin_loop();
        s += 1;
    }
    let mut i = 0usize;
    while i < PMM_TEST_N {
        AP_PMM_FRAMES[i] = crate::mm::alloc_frame().unwrap_or(0);
        i += 1;
    }
}

// Cross-CPU kernel-heap contention self-test (full-os Part I.3, kernel-wide
// locking / slice 5b). The AP and BSP allocate a batch of heap blocks CONCURRENTLY
// (barrier-released) and each stamps its OWN pattern into every block; the BSP then
// verifies all blocks are non-null, distinct, and still hold the right pattern -- a
// block handed to both CPUs (or overlapping) would have one CPU's pattern clobbered,
// meaning the heap lock failed to serialize the free-list operations.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const HEAP_TEST_N: usize = 128;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const HEAP_TEST_SZ: usize = 64;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const HEAP_TEST_ALIGN: usize = 16;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const HEAP_PAT_AP: u64 = 0xAAAA_AAAA_AAAA_AAAA;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
const HEAP_PAT_BSP: u64 = 0xBBBB_BBBB_BBBB_BBBB;
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static mut AP_HEAP_PTRS: [usize; HEAP_TEST_N] = [0; HEAP_TEST_N];
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static HEAP_TEST_AP_READY: AtomicU64 = AtomicU64::new(0);
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static HEAP_TEST_GO: AtomicU64 = AtomicU64::new(0);

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn heap_test_layout() -> core::alloc::Layout {
    core::alloc::Layout::from_size_align_unchecked(HEAP_TEST_SZ, HEAP_TEST_ALIGN)
}

/// AP side of the heap contention test (work kind 4): reach the barrier, wait for
/// the BSP's go, then allocate HEAP_TEST_N heap blocks (stamping the AP pattern into
/// each) into the shared pointer array -- concurrently with the BSP doing the same.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn heap_test_ap_alloc() {
    let layout = heap_test_layout();
    HEAP_TEST_AP_READY.store(1, Ordering::Release);
    let mut s = 0u64;
    while HEAP_TEST_GO.load(Ordering::Acquire) == 0 && s < 500_000_000 {
        core::hint::spin_loop();
        s += 1;
    }
    let mut i = 0usize;
    while i < HEAP_TEST_N {
        let p = alloc::alloc::alloc(layout);
        AP_HEAP_PTRS[i] = p as usize;
        if !p.is_null() {
            *(p as *mut u64) = HEAP_PAT_AP;
        }
        i += 1;
    }
}

/// Run a dispatched work item on the current CPU. Kind 1 = sum 1..=arg, computed
/// iteratively (a real workload the dispatcher can independently check). Kind 3 =
/// the AP side of the PMM contention self-test; kind 4 = the AP side of the heap
/// contention self-test.
unsafe fn run_work(kind: u64, arg: u64) -> u64 {
    match kind {
        1 => {
            let mut sum = 0u64;
            let mut i = 1u64;
            while i <= arg {
                sum = sum.wrapping_add(i);
                i += 1;
            }
            sum
        }
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        3 => {
            pmm_test_ap_alloc();
            0
        }
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        4 => {
            heap_test_ap_alloc();
            0
        }
        _ => 0,
    }
}

/// Poll the work mailbox on an AP: if an item is pending, claim it atomically
/// (so exactly one AP runs it), execute it, and publish the result. Called from
/// the AP park loop after each wake.
unsafe fn ap_poll_work() {
    let gen = WORK_GEN.load(Ordering::Acquire);
    if gen != 0
        && WORK_CLAIM
            .compare_exchange(gen, 0, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
    {
        let kind = WORK_KIND.load(Ordering::Acquire);
        let arg = WORK_ARG.load(Ordering::Acquire);
        // Kind 2 = run a ring-3 user task on this AP (SMP capstone). It does not
        // return here: it enters ring 3 and later resumes in ap_user_done, which
        // republishes completion and re-enters the park loop.
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        {
            if kind == 2 {
                ap_run_user_task(gen, arg);
            }
        }
        let r = run_work(kind, arg);
        WORK_RESULT.store(r, Ordering::Release);
        WORK_DONE.store(gen, Ordering::Release);
    }
}

/// Dispatch a work item from the BSP and wait (bounded) for an AP to finish it.
/// Returns the result, or None if no AP is online or the wait timed out. The AP
/// wakes on its periodic LAPIC timer, polls the mailbox, claims, and runs.
unsafe fn smp_dispatch_work(kind: u64, arg: u64) -> Option<u64> {
    if SMP_AP_COUNT.load(Ordering::Acquire) == 0 {
        return None;
    }
    let gen = WORK_GEN.load(Ordering::Acquire).wrapping_add(1).max(1);
    WORK_KIND.store(kind, Ordering::Release);
    WORK_ARG.store(arg, Ordering::Release);
    WORK_DONE.store(0, Ordering::Release);
    WORK_CLAIM.store(gen, Ordering::Release);
    WORK_GEN.store(gen, Ordering::Release); // publish LAST so the item is consistent
    let mut spins = 0u64;
    while WORK_DONE.load(Ordering::Acquire) != gen && spins < 200_000_000 {
        core::hint::spin_loop();
        spins += 1;
    }
    if WORK_DONE.load(Ordering::Acquire) == gen {
        Some(WORK_RESULT.load(Ordering::Acquire))
    } else {
        None
    }
}

/// Dispatch a work item WITHOUT waiting (the split of `smp_dispatch_work`): publish
/// the item and return its generation so the BSP can run concurrently and join later
/// with `smp_join`. Returns 0 if no AP is online (nothing will run it).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn smp_dispatch_async(kind: u64, arg: u64) -> u64 {
    if SMP_AP_COUNT.load(Ordering::Acquire) == 0 {
        return 0;
    }
    let gen = WORK_GEN.load(Ordering::Acquire).wrapping_add(1).max(1);
    WORK_KIND.store(kind, Ordering::Release);
    WORK_ARG.store(arg, Ordering::Release);
    WORK_DONE.store(0, Ordering::Release);
    WORK_CLAIM.store(gen, Ordering::Release);
    WORK_GEN.store(gen, Ordering::Release); // publish LAST so the item is consistent
    gen
}

/// Wait (bounded) for a previously `smp_dispatch_async`'d item to finish.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn smp_join(gen: u64) -> Option<u64> {
    if gen == 0 {
        return None;
    }
    let mut spins = 0u64;
    while WORK_DONE.load(Ordering::Acquire) != gen && spins < 200_000_000 {
        core::hint::spin_loop();
        spins += 1;
    }
    if WORK_DONE.load(Ordering::Acquire) == gen {
        Some(WORK_RESULT.load(Ordering::Acquire))
    } else {
        None
    }
}

// ---- Per-CPU run queues (full-os guide Part I.3, SMP scheduler). The single
// work mailbox above dispatches ONE item to whichever AP grabs it; a real SMP
// scheduler instead gives EACH CPU its own run queue that it drains
// independently and concurrently. This implements that data structure: the BSP
// enqueues a batch of work to a chosen CPU's queue, each AP drains ONLY its own
// queue (reached through its GS-based per-CPU slot, no cross-CPU locking) and
// accumulates per-CPU state. v1 runs kernel work items; migrating actual R4
// tasks onto these queues with a per-CPU `current` is the remaining scheduler
// work. (Ungated: exercised on both the base -smp 4 lane and the -smp 2 go lane.)
const RQ_LEN: usize = 8;
static mut AP_RQ_KIND: [[u64; RQ_LEN]; MAX_CPUS] = [[0; RQ_LEN]; MAX_CPUS];
static mut AP_RQ_ARG: [[u64; RQ_LEN]; MAX_CPUS] = [[0; RQ_LEN]; MAX_CPUS];
static AP_RQ_COUNT: [AtomicU64; MAX_CPUS] = [const { AtomicU64::new(0) }; MAX_CPUS];
static AP_RQ_DONE: [AtomicU64; MAX_CPUS] = [const { AtomicU64::new(0) }; MAX_CPUS];
static AP_RQ_SUM: [AtomicU64; MAX_CPUS] = [const { AtomicU64::new(0) }; MAX_CPUS];

/// Drain THIS CPU's run queue (reached via its GS-based slot): run each newly
/// enqueued item on this core and fold the result into the per-CPU accumulator.
/// Called from the AP park loop after each wake. Lock-free: only this CPU writes
/// its own DONE/SUM, only the BSP writes its KIND/ARG/COUNT (before publishing).
unsafe fn ap_poll_rq() {
    let slot: u64;
    core::arch::asm!("mov {}, gs:[0]", out(reg) slot, options(nostack));
    let slot = slot as usize;
    if slot == 0 || slot >= MAX_CPUS {
        return;
    }
    let count = AP_RQ_COUNT[slot].load(Ordering::Acquire);
    let mut done = AP_RQ_DONE[slot].load(Ordering::Relaxed);
    while done < count {
        let i = (done as usize) % RQ_LEN;
        let r = run_work(AP_RQ_KIND[slot][i], AP_RQ_ARG[slot][i]);
        AP_RQ_SUM[slot].fetch_add(r, Ordering::AcqRel);
        done += 1;
        AP_RQ_DONE[slot].store(done, Ordering::Release);
    }
}

/// Enqueue a batch of (kind, arg) work items to CPU `cpu`'s run queue, resetting
/// its done/sum accumulators. The item count is published LAST so the consumer
/// never sees a partially-filled queue.
unsafe fn rq_enqueue(cpu: usize, items: &[(u64, u64)]) {
    if cpu == 0 || cpu >= MAX_CPUS {
        return;
    }
    // QUIESCE the queue FIRST by publishing COUNT=0, before resetting DONE/SUM or
    // rewriting KIND/ARG. This matters when re-enqueueing onto a LIVE queue (e.g.
    // the affinity test reusing a queue the run-queue test left at COUNT=DONE=3)
    // while its consumer keeps polling: ap_poll_rq reads COUNT then DONE. With
    // COUNT written LAST, on x86-TSO the consumer could observe the new DONE=0
    // while still seeing the old COUNT=3 and drain stale items. With COUNT=0
    // first, DONE=0's visibility implies COUNT=0's (stores propagate in program
    // order), so the consumer sees `done < 0` (false) and waits for the final
    // COUNT publish — after which all prior stores are already visible.
    AP_RQ_COUNT[cpu].store(0, Ordering::Release);
    AP_RQ_DONE[cpu].store(0, Ordering::Release);
    AP_RQ_SUM[cpu].store(0, Ordering::Release);
    let n = items.len().min(RQ_LEN);
    let mut i = 0;
    while i < n {
        AP_RQ_KIND[cpu][i] = items[i].0;
        AP_RQ_ARG[cpu][i] = items[i].1;
        i += 1;
    }
    AP_RQ_COUNT[cpu].store(n as u64, Ordering::Release);
}

/// Per-CPU run-queue self-test (full-os guide Part I.3): give every online AP its
/// own 3-item queue (sum 1..=100, 1..=200, 1..=300 = 70300), then confirm each AP
/// drained its OWN queue concurrently and accumulated exactly that per-CPU total.
/// Returns true if every AP's queue produced the right sum.
unsafe fn ap_runqueue_selftest() -> bool {
    let online = SMP_AP_COUNT.load(Ordering::Acquire) as usize;
    if online == 0 {
        return false;
    }
    let items = [(1u64, 100u64), (1, 200), (1, 300)];
    let expect = 5050u64 + 20100 + 45150; // 70300
    let mut slot = 1usize;
    while slot <= online && slot < MAX_CPUS {
        rq_enqueue(slot, &items);
        slot += 1;
    }
    // Wait (bounded) for every AP to drain its own queue.
    let mut spins = 0u64;
    loop {
        let mut all = true;
        let mut s = 1usize;
        while s <= online && s < MAX_CPUS {
            if AP_RQ_DONE[s].load(Ordering::Acquire) != items.len() as u64 {
                all = false;
                break;
            }
            s += 1;
        }
        if all || spins >= 200_000_000 {
            break;
        }
        spins += 1;
        core::hint::spin_loop();
    }
    let mut ok = true;
    let mut s = 1usize;
    while s <= online && s < MAX_CPUS {
        if AP_RQ_DONE[s].load(Ordering::Acquire) != items.len() as u64
            || AP_RQ_SUM[s].load(Ordering::Acquire) != expect
        {
            ok = false;
        }
        s += 1;
    }
    ok
}

/// PMM SMP-safety self-test (full-os Part I.3, kernel-wide locking / slice 5): the
/// BSP and one AP allocate a batch of physical frames CONCURRENTLY -- released
/// together by a barrier so the two batches genuinely overlap in time -- and the
/// BSP verifies they are DISJOINT (no frame handed to both CPUs) and that freeing
/// everything restores the free-frame count. A frame appearing in both batches, or
/// a leaked/duplicated count, would mean the PMM lock failed to serialize the
/// allocator's bitmap read-modify-write across cores. Returns true on success;
/// returns false (skips) if no AP is online or the AP never reached the barrier.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn pmm_smp_selftest() -> bool {
    if SMP_AP_COUNT.load(Ordering::Acquire) == 0 {
        return false;
    }
    PMM_TEST_AP_READY.store(0, Ordering::Release);
    PMM_TEST_GO.store(0, Ordering::Release);
    let mut k = 0usize;
    while k < PMM_TEST_N {
        AP_PMM_FRAMES[k] = 0;
        k += 1;
    }
    let baseline = crate::mm::free_frames();
    // Dispatch the AP's batch-alloc (work kind 3); it runs to the barrier and waits.
    let gen = smp_dispatch_async(3, 0);
    if gen == 0 {
        return false; // no AP claimed (none online)
    }
    // Wait for the AP to reach the barrier so the release maximizes overlap.
    let mut s = 0u64;
    while PMM_TEST_AP_READY.load(Ordering::Acquire) == 0 && s < 500_000_000 {
        core::hint::spin_loop();
        s += 1;
    }
    if PMM_TEST_AP_READY.load(Ordering::Acquire) == 0 {
        let _ = smp_join(gen);
        return false; // AP never arrived (degraded boot)
    }
    // Release both cores, then allocate the BSP's batch concurrently with the AP.
    PMM_TEST_GO.store(1, Ordering::Release);
    let mut bsp_frames = [0u64; PMM_TEST_N];
    let mut i = 0usize;
    while i < PMM_TEST_N {
        bsp_frames[i] = crate::mm::alloc_frame().unwrap_or(0);
        i += 1;
    }
    let _ = smp_join(gen); // AP finished its batch
    // Every frame must be a real allocation (alloc never returned 0/None).
    let mut all_alloced = true;
    let mut a = 0usize;
    while a < PMM_TEST_N {
        if bsp_frames[a] == 0 || AP_PMM_FRAMES[a] == 0 {
            all_alloced = false;
        }
        a += 1;
    }
    // Cross-CPU disjointness: alloc_frame returns frames distinct WITHIN one CPU's
    // sequence by construction, so the lock's job is purely cross-core -- no BSP
    // frame may equal any AP frame. O(N^2) over N=256 (~65k compares) at boot.
    let mut disjoint = true;
    let mut bi = 0usize;
    while bi < PMM_TEST_N {
        let bf = bsp_frames[bi];
        let mut aj = 0usize;
        while aj < PMM_TEST_N {
            if bf != 0 && bf == AP_PMM_FRAMES[aj] {
                disjoint = false;
            }
            aj += 1;
        }
        bi += 1;
    }
    // Return every frame and confirm the pool is conserved (no leak / double-free).
    let mut f = 0usize;
    while f < PMM_TEST_N {
        if bsp_frames[f] != 0 {
            crate::mm::free_frame(bsp_frames[f]);
        }
        if AP_PMM_FRAMES[f] != 0 {
            crate::mm::free_frame(AP_PMM_FRAMES[f]);
        }
        f += 1;
    }
    let conserved = crate::mm::free_frames() == baseline;
    all_alloced && disjoint && conserved
}

/// Kernel-heap SMP-safety self-test (full-os Part I.3, kernel-wide locking / slice
/// 5b): the BSP and one AP allocate a batch of heap blocks CONCURRENTLY (barrier-
/// released so the two batches overlap in time), each stamping its OWN pattern into
/// every block, and the BSP verifies every block is non-null, distinct, and STILL
/// holds the correct pattern. A block handed to both CPUs (or two overlapping
/// blocks) would have one CPU's pattern overwritten by the other -- proof the heap
/// lock failed to serialize the free-list. All blocks are freed afterwards. Returns
/// false (skips) if no AP is online or the AP never reaches the barrier.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn heap_smp_selftest() -> bool {
    if SMP_AP_COUNT.load(Ordering::Acquire) == 0 {
        return false;
    }
    let layout = heap_test_layout();
    HEAP_TEST_AP_READY.store(0, Ordering::Release);
    HEAP_TEST_GO.store(0, Ordering::Release);
    let mut k = 0usize;
    while k < HEAP_TEST_N {
        AP_HEAP_PTRS[k] = 0;
        k += 1;
    }
    let gen = smp_dispatch_async(4, 0);
    if gen == 0 {
        return false;
    }
    let mut s = 0u64;
    while HEAP_TEST_AP_READY.load(Ordering::Acquire) == 0 && s < 500_000_000 {
        core::hint::spin_loop();
        s += 1;
    }
    if HEAP_TEST_AP_READY.load(Ordering::Acquire) == 0 {
        let _ = smp_join(gen);
        return false;
    }
    // Release both, then allocate + stamp the BSP's batch concurrently with the AP.
    HEAP_TEST_GO.store(1, Ordering::Release);
    let mut bsp_ptrs = [0usize; HEAP_TEST_N];
    let mut i = 0usize;
    while i < HEAP_TEST_N {
        let p = alloc::alloc::alloc(layout);
        bsp_ptrs[i] = p as usize;
        if !p.is_null() {
            *(p as *mut u64) = HEAP_PAT_BSP;
        }
        i += 1;
    }
    let _ = smp_join(gen);
    // Every allocation succeeded (a corrupt free list would yield null or a crash).
    let mut all_alloced = true;
    let mut a = 0usize;
    while a < HEAP_TEST_N {
        if bsp_ptrs[a] == 0 || AP_HEAP_PTRS[a] == 0 {
            all_alloced = false;
        }
        a += 1;
    }
    // Pattern integrity: after BOTH cores wrote, every block must still hold its own
    // CPU's pattern. If any two blocks (cross- or same-CPU) aliased/overlapped, one
    // stamp clobbered the other -> a mismatch here. This catches double hand-out and
    // overlap without an explicit O(N^2) address-range comparison.
    let mut patterns_ok = true;
    let mut bi = 0usize;
    while bi < HEAP_TEST_N {
        if bsp_ptrs[bi] != 0 && *(bsp_ptrs[bi] as *const u64) != HEAP_PAT_BSP {
            patterns_ok = false;
        }
        if AP_HEAP_PTRS[bi] != 0 && *(AP_HEAP_PTRS[bi] as *const u64) != HEAP_PAT_AP {
            patterns_ok = false;
        }
        bi += 1;
    }
    // Cross-CPU pointer distinctness: no BSP block may share an address with an AP
    // block (the heap must never hand the same block to two cores). Within one CPU's
    // sequence the allocator already returns distinct blocks by construction.
    let mut distinct = true;
    let mut x = 0usize;
    while x < HEAP_TEST_N {
        let bp = bsp_ptrs[x];
        let mut y = 0usize;
        while y < HEAP_TEST_N {
            if bp != 0 && bp == AP_HEAP_PTRS[y] {
                distinct = false;
            }
            y += 1;
        }
        x += 1;
    }
    // Return every block to the heap.
    let mut f = 0usize;
    while f < HEAP_TEST_N {
        if bsp_ptrs[f] != 0 {
            alloc::alloc::dealloc(bsp_ptrs[f] as *mut u8, layout);
        }
        if AP_HEAP_PTRS[f] != 0 {
            alloc::alloc::dealloc(AP_HEAP_PTRS[f] as *mut u8, layout);
        }
        f += 1;
    }
    all_alloced && patterns_ok && distinct
}

/// Per-CPU affinity + load-distribution self-test (full-os guide Part I.3, SMP
/// scheduler load balancing): unlike `ap_runqueue_selftest` (every AP gets the
/// SAME queue), this gives each CPU a DISTINCT workload and verifies each core
/// drained exactly ITS OWN work — proving the BSP can ROUTE specific work to a
/// specific core (affinity), the basis for load balancing across the per-CPU run
/// queues. Also checks the grand total, i.e. that the whole batch was distributed
/// across the cores with nothing lost or double-run. Returns true on full match.
unsafe fn ap_affinity_selftest() -> bool {
    let online = SMP_AP_COUNT.load(Ordering::Acquire) as usize;
    if online == 0 {
        return false;
    }
    // Each CPU's two-item queue is keyed off its slot, so no two cores share a
    // workload — a core running the wrong queue would produce the wrong sum.
    let mut slot = 1usize;
    while slot <= online && slot < MAX_CPUS {
        let a = (slot as u64) * 100;
        let b = (slot as u64) * 50;
        rq_enqueue(slot, &[(1u64, a), (1u64, b)]);
        slot += 1;
    }
    // Wait (bounded) for every AP to drain its own (2-item) queue.
    let mut spins = 0u64;
    loop {
        let mut all = true;
        let mut s = 1usize;
        while s <= online && s < MAX_CPUS {
            if AP_RQ_DONE[s].load(Ordering::Acquire) != 2 {
                all = false;
                break;
            }
            s += 1;
        }
        if all || spins >= 200_000_000 {
            break;
        }
        spins += 1;
        core::hint::spin_loop();
    }
    // Verify each CPU produced ITS OWN distinct sum, and accumulate the grand
    // total to confirm the whole batch was distributed across the cores.
    let mut ok = true;
    let mut got_total = 0u64;
    let mut want_total = 0u64;
    let mut s = 1usize;
    while s <= online && s < MAX_CPUS {
        let a = (s as u64) * 100;
        let b = (s as u64) * 50;
        let expect = a * (a + 1) / 2 + b * (b + 1) / 2; // T(a)+T(b)
        want_total += expect;
        let sum = AP_RQ_SUM[s].load(Ordering::Acquire);
        got_total += sum;
        if AP_RQ_DONE[s].load(Ordering::Acquire) != 2 || sum != expect {
            ok = false;
        }
        s += 1;
    }
    ok && got_total == want_total
}

// ---- SMP scheduler capstone: run a ring-3 USER task on an application
// processor (full-os guide Part I.3). The primitives above (IPI, per-CPU LAPIC
// timer, per-CPU GS, cross-CPU work dispatch, TLB shootdown) made an AP run
// dispatched KERNEL work; this makes an AP enter ring 3, execute user code on
// its own core, take the ring-3->ring-0 trap onto its OWN per-CPU kernel stack
// (via its own TSS rsp0), and report a result the BSP verifies.
//
// Flow: the BSP builds a private address space holding the user code + a stack
// (the same mm path spawned apps use), publishes it, and dispatches work kind 2.
// An AP claims it, sets its per-CPU `current` task (gs:[16] — the scheduler's
// bookkeeping), loads that CR3, and `iretq`s to ring 3 with the arg in RDI.
// The user task issues TWO REAL syscalls — `int 0x80` sys_time_now — exercising
// the full ring-3->ring-0->ring-3 syscall path on the AP's own per-CPU TSS rsp0
// (serviced on the second core), then reports their tick delta (== 1) and arg*2+1
// via `int 0x81`; ap_user_trap reads the per-CPU `current` back through GS, records
// the result, restores the kernel CR3, and trampolines the AP back into kernel
// code (ap_user_done) on its own kernel stack — mirroring the m3 ring-3->kernel
// return — which republishes completion and resumes normal AP polling. The BSP
// verifies the result, that an AP (slot >= 1) ran it, and that the per-CPU
// `current` round-tripped.

// Ring-3 payload (position-independent, touches no user memory so every page is
// premapped and it never demand-faults on the AP). It issues TWO REAL syscalls —
// `int 0x80` sys_time_now (op 10) — exercising the full ring-3->ring-0->ring-3
// syscall path on the AP's own per-CPU TSS rsp0, and reports the delta of the two
// monotonic ticks (must be exactly 1 — proof real kernel code ran for each call)
// alongside 2*arg+1. sys_time_now is used (not sys_debug_write) because the latter
// mirrors to the framebuffer console, which lives under PML4[0] — the half
// address_space_create replaces — so it is absent from this AP address space.
//   48 89 FB              mov rbx, rdi    ; save arg (arrives in RDI)
//   B8 0A 00 00 00        mov eax, 10     ; nr 10 = sys_time_now
//   CD 80                 int 0x80        ; rax = t1   (REAL syscall #1)
//   48 89 C1              mov rcx, rax    ; save t1
//   B8 0A 00 00 00        mov eax, 10     ; nr 10 = sys_time_now
//   CD 80                 int 0x80        ; rax = t2   (REAL syscall #2)
//   48 29 C8              sub rax, rcx    ; rax = t2 - t1  (== 1)
//   48 89 C6              mov rsi, rax    ; report delta in RSI
//   48 89 DF              mov rdi, rbx    ; restore arg
//   48 01 FF              add rdi, rdi    ; rdi = 2*arg
//   48 83 C7 01           add rdi, 1      ; rdi = 2*arg+1  (report in RDI)
//   CD 81                 int 0x81        ; report to ap_user_trap
//   EB FE               1: jmp 1b        ; unreachable (trampoline takes over)
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_USER_CODE: [u8; 40] = [
    0x48, 0x89, 0xFB, 0xB8, 0x0A, 0x00, 0x00, 0x00, 0xCD, 0x80, 0x48, 0x89, 0xC1, 0xB8, 0x0A,
    0x00, 0x00, 0x00, 0xCD, 0x80, 0x48, 0x29, 0xC8, 0x48, 0x89, 0xC6, 0x48, 0x89, 0xDF, 0x48,
    0x01, 0xFF, 0x48, 0x83, 0xC7, 0x01, 0xCD, 0x81, 0xEB, 0xFE,
];

// Ring-3 payload for the REAL-R4-task migration test. It issues two genuine, real
// (not test-only) syscalls that both resolve the task's OWN slot via per-CPU current
// on the AP: a WRITE -- sys_sysinfo op 16, which bumps R4_TASKS[r4_current_smp()]
// .yield_count -- then a READ -- getuid (sys_proc_ctl id 51 op 3), R4_TASKS
// [r4_current_smp()].uid. The BSP checks the write landed on the migrated slot (its
// yield_count) and the read returned the slot's sentinel uid (reported in RSI):
//   48 89 FB              mov rbx, rdi    ; save arg
//   B8 3D 00 00 00        mov eax, 61     ; nr 61 = sys_sysinfo
//   BF 10 00 00 00        mov edi, 16     ; op 16 = bump my yield_count (WRITE)
//   31 F6 / 31 D2         xor esi/edx
//   CD 80                 int 0x80
//   B8 33 00 00 00        mov eax, 51     ; nr 51 = sys_proc_ctl
//   BF 03 00 00 00        mov edi, 3      ; op 3 = getuid (READ)
//   31 F6 / 31 D2         xor esi/edx
//   CD 80                 int 0x80        ; rax = R4_TASKS[per-cpu current].uid
//   48 89 C6              mov rsi, rax    ; report resolved uid in RSI
//   48 89 DF / 48 01 FF / 48 83 C7 01     ; rdi = 2*arg+1 (ran-in-ring3 proof)
//   CD 81 / EB FE
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_R4_CODE: [u8; 52] = [
    0x48, 0x89, 0xFB, 0xB8, 0x3D, 0x00, 0x00, 0x00, 0xBF, 0x10, 0x00, 0x00, 0x00, 0x31, 0xF6,
    0x31, 0xD2, 0xCD, 0x80, 0xB8, 0x33, 0x00, 0x00, 0x00, 0xBF, 0x03, 0x00, 0x00, 0x00, 0x31,
    0xF6, 0x31, 0xD2, 0xCD, 0x80, 0x48, 0x89, 0xC6, 0x48, 0x89, 0xDF, 0x48, 0x01, 0xFF, 0x48,
    0x83, 0xC7, 0x01, 0xCD, 0x81, 0xEB, 0xFE,
];

// Rendezvous state for the concurrent-execution proof: a ring-3 task running on an
// AP and the BSP must both be live at the same instant to complete it (0 = idle,
// 1 = AP arrived, 2 = BSP acknowledged). If the two were NOT concurrent the AP's
// in-kernel wait would time out, so a clean rendezvous is proof of simultaneity.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static SMP_RV: AtomicU64 = AtomicU64::new(0);

// Ring-3 payload for the concurrency proof: a single syscall (sys_sysinfo op 15 ->
// smp_rendezvous_ap) that, on the AP, signals arrival and spins for the BSP's ack,
// returning 0xAC on success; the task then reports that code. Same shape as
// AP_R4_CODE but op 15:
//   48 89 FB              mov rbx, rdi
//   B8 3D 00 00 00        mov eax, 61     ; sys_sysinfo
//   BF 0F 00 00 00        mov edi, 15     ; op 15 = SMP rendezvous
//   31 F6 / 31 D2         xor esi/edx
//   CD 80                 int 0x80        ; rax = 0xAC on a completed rendezvous
//   48 89 C6              mov rsi, rax
//   48 89 DF / 48 01 FF / 48 83 C7 01     ; rdi = 2*arg+1
//   CD 81 / EB FE
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_RV_CODE: [u8; 36] = [
    0x48, 0x89, 0xFB, 0xB8, 0x3D, 0x00, 0x00, 0x00, 0xBF, 0x0F, 0x00, 0x00, 0x00, 0x31, 0xF6,
    0x31, 0xD2, 0xCD, 0x80, 0x48, 0x89, 0xC6, 0x48, 0x89, 0xDF, 0x48, 0x01, 0xFF, 0x48, 0x83,
    0xC7, 0x01, 0xCD, 0x81, 0xEB, 0xFE,
];

/// AP side of the concurrency rendezvous (invoked by `sys_sysinfo` op 15 while the
/// migrated task runs in ring 3 on an application processor): publish arrival, then
/// spin (bounded) for the BSP's acknowledgement. Returns 0xAC if the BSP answered
/// (proof both CPUs were executing at the same time), 0xFA on timeout. On the BSP it
/// is a no-op sentinel (0) -- only the AP performs the rendezvous.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn smp_rendezvous_ap() -> u64 {
    if is_bsp() {
        return 0;
    }
    SMP_RV.store(1, Ordering::Release); // AP has arrived
    let mut spins = 0u64;
    while SMP_RV.load(Ordering::Acquire) != 2 && spins < 200_000_000 {
        core::hint::spin_loop();
        spins += 1;
    }
    if SMP_RV.load(Ordering::Acquire) == 2 {
        0xAC
    } else {
        0xFA
    }
}

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_USER_TASKID: AtomicU64 = AtomicU64::new(0); // id of the task migrated to the AP
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_USER_CURRENT: AtomicU64 = AtomicU64::new(0); // per-CPU `current` the AP read back via GS
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_USER_SYSRET: AtomicU64 = AtomicU64::new(0); // delta of the two time_now syscalls the AP ran
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_USER_CR3: AtomicU64 = AtomicU64::new(0); // user address space to run
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_USER_ENTRY: AtomicU64 = AtomicU64::new(0); // ring-3 entry VA
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_USER_SP: AtomicU64 = AtomicU64::new(0); // ring-3 stack top
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_USER_RESULT: AtomicU64 = AtomicU64::new(0); // value the task reported
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_USER_CPU: AtomicU64 = AtomicU64::new(0); // gs:[0] of the AP that ran it
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_USER_GEN: AtomicU64 = AtomicU64::new(0); // work gen, for WORK_DONE
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_SAVED_CR3: AtomicU64 = AtomicU64::new(0); // kernel CR3 to restore

/// Run a ring-3 user task on the current AP. Never returns to the caller: it
/// enters ring 3 and later resumes in `ap_user_done`. Requires this AP to own a
/// TSS (per-CPU slot < MAX_TSS, recorded in gs:[0]); without one it cannot take
/// the ring-3->ring-0 transition, so it reports a sentinel and resumes polling.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn ap_run_user_task(gen: u64, arg: u64) -> ! {
    let slot: u64;
    core::arch::asm!("mov {}, gs:[0]", out(reg) slot, options(nostack));
    if slot == 0 || slot >= crate::arch_x86::MAX_TSS as u64 {
        AP_USER_RESULT.store(u64::MAX, Ordering::Release);
        AP_USER_CPU.store(slot, Ordering::Release);
        WORK_RESULT.store(u64::MAX, Ordering::Release);
        WORK_DONE.store(gen, Ordering::Release);
        loop {
            ap_poll_work();
            ap_poll_rq();
            core::arch::asm!("sti; hlt", options(nomem, nostack));
        }
    }
    AP_USER_GEN.store(gen, Ordering::Release);
    // Set THIS CPU's `current` task through its own GS base (gs:[16]) — the exact
    // per-CPU bookkeeping a scheduler does when it dispatches a task to a core.
    // Written via gs: (not PERCPU[slot] directly) so a wrong GS base would land in
    // the wrong slot and the BSP's read-back check would fail.
    let tid = AP_USER_TASKID.load(Ordering::Acquire);
    core::arch::asm!("mov qword ptr gs:[16], {v}", v = in(reg) tid, options(nostack));
    let kcr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) kcr3, options(nomem, nostack));
    AP_SAVED_CR3.store(kcr3, Ordering::Release);
    let ucr3 = AP_USER_CR3.load(Ordering::Acquire);
    let entry = AP_USER_ENTRY.load(Ordering::Acquire);
    let usp = AP_USER_SP.load(Ordering::Acquire);
    // Live-scheduler preemption (slice 4): launch the task PREEMPTIBLE (IF=1) so
    // this AP's periodic LAPIC timer can land mid-task and the kernel can preempt
    // it. Only when SMP_PREEMPT_MODE is set (the ap_preempt_selftest window); the
    // capstone / migrate / concurrent / plain-live-sched tasks keep IF=0 (run to
    // completion, no timer mid-flight), so this is transparent to them.
    let preempt = SMP_PREEMPT_MODE.load(Ordering::Acquire) != 0;
    core::arch::asm!("mov cr3, {}", in(reg) ucr3, options(nostack));
    if preempt {
        crate::arch_x86::enter_ring3_with_arg_preempt(entry, usp, arg);
    } else {
        crate::arch_x86::enter_ring3_with_arg(entry, usp, arg);
    }
}

/// AP user-task report handler (trap_handler vector 0x81, ring 3 only). Records
/// the reported value (RDI) and the running AP's slot, restores the kernel CR3,
/// and rewrites the trap frame so the `iretq` resumes `ap_user_done` in ring 0
/// on THIS AP's own kernel stack.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn ap_user_trap(frame: *mut u64) {
    let reported = *frame.add(9); // RDI = 2*arg+1
    let sysret = *frame.add(10); // RSI = delta of the two sys_time_now ticks (== 1)
    AP_USER_SYSRET.store(sysret, Ordering::Release);
    let slot: u64;
    core::arch::asm!("mov {}, gs:[0]", out(reg) slot, options(nostack));
    // Read back THIS CPU's `current` task through its own GS base and publish it,
    // then clear it (the task is leaving the core). Confirms the per-CPU current
    // set in ap_run_user_task round-tripped on the same AP via gs:[16].
    let cur: u64;
    core::arch::asm!("mov {}, gs:[16]", out(reg) cur, options(nostack));
    AP_USER_CURRENT.store(cur, Ordering::Release);
    core::arch::asm!("mov qword ptr gs:[16], 0", options(nostack));
    let kcr3 = AP_SAVED_CR3.load(Ordering::Acquire);
    if kcr3 != 0 {
        core::arch::asm!("mov cr3, {}", in(reg) kcr3, options(nostack));
    }
    AP_USER_RESULT.store(reported, Ordering::Release);
    AP_USER_CPU.store(slot, Ordering::Release);
    let kstack = crate::arch_x86::ap_kstack_top(slot as usize);
    *frame.add(17) = ap_user_done as *const () as u64; // RIP
    *frame.add(18) = 0x08; // CS (ring 0)
    *frame.add(19) = 0x002; // RFLAGS (IF clear)
    *frame.add(20) = kstack; // RSP (this AP's kernel stack)
    *frame.add(21) = 0x10; // SS (ring 0 data)
}

/// Kernel continuation after an AP's ring-3 user task returns: publish
/// completion (the BSP's smp_dispatch_work is waiting on WORK_DONE), then resume
/// normal AP duty.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
extern "C" fn ap_user_done() -> ! {
    unsafe {
        WORK_RESULT.store(AP_USER_RESULT.load(Ordering::Acquire), Ordering::Release);
        WORK_DONE.store(AP_USER_GEN.load(Ordering::Acquire), Ordering::Release);
        // Live-scheduler mode: the task that just finished was pulled from the run
        // set, so retire its slot (under the run-queue lock) and count it -- this is
        // how the AP signals the BSP that an autonomously-scheduled task completed.
        // AP_USER_CURRENT is the tid ap_user_trap read back from this CPU's gs:[16].
        if SMP_LIVE_MODE.load(Ordering::Acquire) != 0 {
            let done_tid = AP_USER_CURRENT.load(Ordering::Acquire) as usize;
            r4_rq_lock();
            if done_tid < crate::R4_MAX_TASKS {
                crate::R4_TASKS[done_tid].state = crate::R4State::Dead;
            }
            r4_rq_unlock();
            SMP_LIVE_RAN.fetch_add(1, Ordering::AcqRel);
        }
        loop {
            ap_poll_work();
            ap_poll_rq();
            ap_pull_r4_task(); // live mode: claim + run the next ready task
            core::arch::asm!("sti; hlt", options(nomem, nostack));
        }
    }
}

/// BSP side of the capstone: build a private address space with the ring-3 user
/// code + a stack, dispatch it to an AP (work kind 2), and verify the AP ran it
/// in ring 3 (result == arg*2+1) on an application processor (slot >= 1).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn ap_user_selftest() -> bool {
    const CODE_VA: u64 = 0x0140_0000; // exec-app window: the loader clears NX here
    const STACK_TOP: u64 = 0x0013_0000;
    const STACK_PAGE: u64 = STACK_TOP - 0x1000;
    const ARG: u64 = 21;
    const TASK_ID: u64 = 0x5A; // the migrated task's id, tracked as the AP's `current`
    let kcr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) kcr3, options(nomem, nostack));
    let ucr3 = match crate::mm::address_space_create(kcr3) {
        Some(p) => p,
        None => return false,
    };
    if !crate::mm::as_copyout(ucr3, CODE_VA, &AP_USER_CODE)
        || !crate::mm::as_map_zeroed(ucr3, STACK_PAGE, 0x1000)
    {
        crate::mm::address_space_release(ucr3);
        return false;
    }
    AP_USER_CR3.store(ucr3, Ordering::Release);
    AP_USER_ENTRY.store(CODE_VA, Ordering::Release);
    AP_USER_SP.store(STACK_TOP, Ordering::Release);
    AP_USER_TASKID.store(TASK_ID, Ordering::Release);
    AP_USER_CURRENT.store(0, Ordering::Release);
    AP_USER_SYSRET.store(0, Ordering::Release);
    let result = smp_dispatch_work(2, ARG);
    let cpu = AP_USER_CPU.load(Ordering::Acquire);
    let cur = AP_USER_CURRENT.load(Ordering::Acquire);
    let sysret = AP_USER_SYSRET.load(Ordering::Acquire);
    // Reclaim the address space ONLY on the success path. There, the AP restored
    // the kernel CR3 (ap_user_trap) *before* publishing WORK_DONE, and we waited
    // on WORK_DONE with Acquire ordering, so the AP is provably off this CR3.
    // On a timeout (None) that happens-before is absent — a claiming AP could
    // still be translating through ucr3 (or a late AP could yet load it) — so we
    // intentionally LEAK rather than free live page tables (a cross-CPU UAF /
    // PMM double-allocation). This boot self-test runs once; the timeout path is
    // unreachable in practice (the AP claims within one LAPIC-timer period).
    if result.is_some() {
        crate::mm::address_space_release(ucr3);
    }
    // Surface the two new facts: (a) the AP serviced REAL syscalls — the delta of
    // its two sys_time_now calls is exactly 1, so real kernel code ran for each
    // int 0x80 on the AP's own core; (b) the per-CPU `current` the AP set + read
    // back through its own GS base matches the dispatched task id.
    serial_write(b"SMP: ap-syscall delta=0x");
    serial_write_hex(sysret);
    serial_write(b"\n");
    serial_write(b"SMP: ap-current=0x");
    serial_write_hex(cur);
    serial_write(b"\n");
    matches!(result, Some(v) if v == ARG * 2 + 1) && cpu >= 1 && cur == TASK_ID && sysret == 1
}

/// LAPIC-timer service routine (from trap_handler vector 241): tick, drive the
/// per-CPU preemption (slice 4), then EOI.
pub(crate) unsafe fn lapic_timer_handler(_frame: *mut u64) {
    AP_TICKS.fetch_add(1, Ordering::SeqCst);
    // Bump THIS CPU's per-CPU tick counter through the GS base: no lock and no
    // CPU-id lookup, because each CPU's GS base points at its own slot. This is
    // the exact access pattern a per-CPU scheduler uses for `current`/run-queue.
    core::arch::asm!("add qword ptr gs:[8], 1", options(nostack));
    // Live per-CPU scheduler slice 4 (preemption): when preempt mode is on and this
    // timer interrupted a RING-3 task on this AP (CS RPL == 3 in the saved frame),
    // count the hit (slice 4a -- proof the AP's clock fires inside a running user
    // task, impossible until the task ran IF=1) and RESCHEDULE the core to another
    // ap_eligible task (slice 4b). This runs in an interrupt gate (IF=0, no
    // re-entrancy); the reschedule takes R4_RQ_LOCK, which is deadlock-safe because
    // the only IF=1 holder of that lock (ap_pull_r4_task) cli's around its claim, so
    // the timer can never fire while it is held on this CPU. EOI happens AFTER any
    // switch (the LAPIC EOI MSR is CR3-independent).
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    {
        if SMP_PREEMPT_MODE.load(Ordering::Acquire) != 0 && (*_frame.add(18) & 3) == 3 {
            AP_PREEMPT_HITS.fetch_add(1, Ordering::AcqRel);
            ap_preempt_reschedule(_frame);
        }
    }
    x2apic_eoi();
}

/// AP timer-preemption reschedule (full-os Part I.3, live per-CPU scheduler slice
/// 4b). Invoked from the LAPIC-timer ISR (interrupt gate, IF=0) when it interrupted
/// a ring-3 task on THIS AP and preempt mode is on. Finds another ap_eligible Ready
/// task, saves the running task's full context into its slot (Ready), claims the
/// next one (Running), and switches this core to it -- a real preemptive context
/// switch driven by the AP's own clock. With nothing else runnable it leaves the
/// current task running.
///
/// Deadlock safety: the only other R4_RQ_LOCK holder that runs with IF=1 is
/// ap_pull_r4_task, which now cli's around its claim, so the timer can never fire
/// while that lock is held on this CPU -- this handler can take R4_RQ_LOCK without
/// risking a self-deadlock. The lock is RELEASED before r4_switch_to so it is never
/// held across the context switch (the switched-to task may itself be preempted and
/// re-enter this path).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn ap_preempt_reschedule(frame: *mut u64) {
    let slot: u64;
    core::arch::asm!("mov {}, gs:[0]", out(reg) slot, options(nostack));
    if slot == 0 || slot >= crate::arch_x86::MAX_TSS as u64 {
        return; // needs a per-CPU TSS to take the ring-3 -> ring-0 transition
    }
    let cur: u64;
    core::arch::asm!("mov {}, gs:[16]", out(reg) cur, options(nostack));
    let cur = cur as usize;
    if cur == 0 || cur >= crate::R4_MAX_TASKS {
        return; // no valid per-CPU current task to preempt
    }
    r4_rq_lock();
    let mut next = usize::MAX;
    let mut j = 1usize;
    while j < crate::R4_MAX_TASKS {
        if j != cur
            && crate::R4_TASKS[j].ap_eligible
            && crate::R4_TASKS[j].state == crate::R4State::Ready
        {
            next = j;
            break;
        }
        j += 1;
    }
    if next != usize::MAX {
        // Save the preempted task (full frame) and requeue it Ready; claim the next
        // one Running so no other CPU takes it between unlock and the switch.
        crate::r4_save_frame(frame, cur);
        crate::R4_TASKS[cur].state = crate::R4State::Ready;
        crate::R4_TASKS[next].state = crate::R4State::Running;
        r4_rq_unlock(); // RELEASE before the context switch
        AP_PREEMPT_SWITCHES.fetch_add(1, Ordering::AcqRel);
        // Load next's full saved frame into *frame + its CR3 + set this AP's
        // per-CPU current; the ISR epilogue's iretq then resumes next in ring 3.
        crate::r4_switch_to(frame, next);
    } else {
        r4_rq_unlock(); // only the current task is runnable -> keep running it
    }
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

/// Number of online CPUs (the BSP plus every AP that checked in). 1 on a
/// uniprocessor lane. Exposed to userspace via sys_sysinfo op 13.
pub fn cpu_count() -> u64 {
    SMP_AP_COUNT.load(Ordering::Acquire) + 1
}

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

// ---- Autonomous SMP scheduling (full-os guide Part I.3): an application processor
// pulls READY R4 tasks from the shared run set itself -- claiming each under a lock
// so no two CPUs run the same task -- and runs them in ring 3 on its own core, with
// no per-task dispatch from the BSP. This is the live-scheduler step beyond the
// BSP-dispatched capstone. Gated behind SMP_LIVE_MODE so the normal scheduler and
// the other SMP self-tests (which run with it off) are untouched.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static R4_RQ_LOCK: AtomicU32 = AtomicU32::new(0); // guards run-set claim/retire
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static SMP_LIVE_MODE: AtomicU64 = AtomicU64::new(0); // 1 = APs may pull
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static SMP_LIVE_RAN: AtomicU64 = AtomicU64::new(0); // tasks completed on APs
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static SMP_LIVE_BASE: AtomicU64 = AtomicU64::new(0); // first run-set slot
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static SMP_LIVE_COUNT: AtomicU64 = AtomicU64::new(0); // run-set slot count
// Live per-CPU scheduler slice 4: when set, an AP pulls its task PREEMPTIBLY
// (ring 3 with IF=1) so this CPU's own periodic LAPIC timer can land while the
// task runs, and the timer handler counts/handles that preemption. 0 = the
// existing run-to-completion behaviour (the other SMP self-tests are untouched).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static SMP_PREEMPT_MODE: AtomicU64 = AtomicU64::new(0);
// Times this AP's LAPIC timer interrupted a ring-3 task on its own core (the
// preemption clock firing during a running user task -- impossible until the
// task runs IF=1).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_PREEMPT_HITS: AtomicU64 = AtomicU64::new(0);
// Times the AP timer actually RESCHEDULED -- saved the running task and switched
// the core to a DIFFERENT ap_eligible task (slice 4b: real time-slicing on the AP).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_PREEMPT_SWITCHES: AtomicU64 = AtomicU64::new(0);

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn r4_rq_lock() {
    while R4_RQ_LOCK
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
}
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn r4_rq_unlock() {
    R4_RQ_LOCK.store(0, Ordering::Release);
}

// ---- Workload-distribution coarse leaf locks (full-os guide Part I.3, SMP
// workload distribution). When an application processor pulls a REAL go-shell task
// from the BSP's live rotation, that task's FS / network / block syscalls touch
// shared single-instance kernel state -- the VFS inode+bitmap cache + journal, the
// TCP/DHCP connection + socket tables, and the single global BLK_DATA_PAGE I/O
// buffer -- concurrently with the BSP (which is also running tasks and PIT-pumping
// the network every tick). Today that state is safe ONLY because every caller runs
// IF=0 on a single core; on SMP two CPUs race the read-modify-writes. These three
// coarse leaf spinlocks serialize each family. They are COARSE (not per-CPU /
// per-object): the workload is light, the sections short, and one auditable lock per
// subsystem keeps a single global acquisition order.
//
// DEADLOCK-FREE GLOBAL ORDER (acquire left->right = outer->inner, release reverse; a
// CPU may take a lock only while it holds none to its right). STORAGE is the INNERMOST
// data lock because both higher layers call DOWN into block I/O:
//     PMM_LOCK < HEAP_LOCK < FS_LOCK < NET_LOCK < STORAGE_LOCK < R4_RQ_LOCK
//   - FS < STORAGE: an FS write holds FS_LOCK across its block I/O (vfs_write ->
//     jwrite -> write_sector -> block_io_dispatch). FS outer, STORAGE inner.
//   - NET < STORAGE: the package-fetch path holds NET_LOCK across its install write
//     (the BSP PIT pump's pkg_fetch_tick -> block_io_dispatch, lib.rs r4_timer_preempt
//     ~4491-4496). NET outer, STORAGE inner. The reverse (disk-then-net) never happens
//     -- block I/O never takes FS or NET.
//   - FS and NET are NEVER co-held (vfs.rs has zero net references; the net pump takes
//     no FS lock), so their relative rank is free -- FS<NET is arbitrary.
//   - STORAGE/FS/NET > HEAP/PMM: I/O, FS and net never allocate while holding their
//     lock (any allocation happens before the lock is taken), so PMM/HEAP are never
//     nested inside them.
//   - R4_RQ_LOCK is the innermost overall: a disjoint scheduler domain (the AP claim/
//     retire + the AP LAPIC-timer reschedule) that never nests with FS/NET/STORAGE --
//     a task is claimed under R4_RQ, which is RELEASED before the task runs its FS/net
//     syscalls. Ranked last so the order is total.
// INTERRUPT-CONTEXT REACH (drives the leaf discipline): the BSP PIT interrupt gate
// (r4_timer_preempt, IF=0) takes NET_LOCK (net_rx_pump) and STORAGE_LOCK (pkg_fetch
// install) in its net-pump section, and R4_RQ_LOCK in a SEPARATE later wake section;
// the AP LAPIC-timer reschedule takes R4_RQ_LOCK. So NET, STORAGE and R4_RQ are all
// reachable from an interrupt handler -- NOT just R4_RQ.
// LEAF DISCIPLINE: take FS/NET/STORAGE with interrupts DISABLED on AP-reachable (IF=1)
// task paths via lock_cli()/unlock_sti(), so this CPU's own timer can't land
// mid-section and either spin forever on a lock the interrupted task holds (the PIT
// pump re-entering net_rx_pump on a BSP task that holds NET_LOCK) or violate the
// order. The interrupt-context holders (the BSP PIT pump, the AP reschedule) already
// run IF=0 in their gate, auto-satisfying this. NEVER hold one across a blocking switch.
//
// Slice 1 defines them UNWIRED (lock_infra_selftest is the only caller); slices 2-5
// wire each into the block / VFS / net / wake paths.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static FS_LOCK: AtomicU32 = AtomicU32::new(0);
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static STORAGE_LOCK: AtomicU32 = AtomicU32::new(0);
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static NET_LOCK: AtomicU32 = AtomicU32::new(0);
// Contention counters: bumped once per failed acquire (a spin where another CPU held
// the lock). Stay 0 on -smp 1 (every acquire uncontended). Read by the capstone's
// "contend fs>0 net>0 blk>0" proof that the tasks were TRULY concurrent.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static FS_CONTENTION: AtomicU64 = AtomicU64::new(0);
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static STORAGE_CONTENTION: AtomicU64 = AtomicU64::new(0);
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static NET_CONTENTION: AtomicU64 = AtomicU64::new(0);

// Each lock's inner single attempt: the step its spinning acquire loops on, factored
// out so the contention bump lives in one place (and so a single-core self-test can
// drive a failed acquire without an infinite spin). Returns true if it took the lock.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
#[inline]
unsafe fn fs_try_acquire() -> bool {
    if FS_LOCK
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        true
    } else {
        FS_CONTENTION.fetch_add(1, Ordering::Relaxed);
        false
    }
}
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
#[inline]
unsafe fn storage_try_acquire() -> bool {
    if STORAGE_LOCK
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        true
    } else {
        STORAGE_CONTENTION.fetch_add(1, Ordering::Relaxed);
        false
    }
}
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
#[inline]
unsafe fn net_try_acquire() -> bool {
    if NET_LOCK
        .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
    {
        true
    } else {
        NET_CONTENTION.fetch_add(1, Ordering::Relaxed);
        false
    }
}

#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn fs_lock() {
    while !fs_try_acquire() {
        core::hint::spin_loop();
    }
}
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn fs_unlock() {
    FS_LOCK.store(0, Ordering::Release);
}
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn storage_lock() {
    while !storage_try_acquire() {
        core::hint::spin_loop();
    }
}
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn storage_unlock() {
    STORAGE_LOCK.store(0, Ordering::Release);
}
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn net_lock() {
    while !net_try_acquire() {
        core::hint::spin_loop();
    }
}
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn net_unlock() {
    NET_LOCK.store(0, Ordering::Release);
}

/// Disable interrupts for a leaf-lock critical section on an AP-reachable (IF=1)
/// path, returning the prior RFLAGS so the matching unlock_sti can restore the
/// caller's interrupt-enable state. Pairs with unlock_sti. Mirrors the
/// cli/save-RFLAGS pattern already in ap_pull_r4_task. Rationale: the BSP PIT gate
/// re-enters net_rx_pump (NET_LOCK) + pkg_fetch (STORAGE_LOCK) and the AP LAPIC-timer
/// reschedule takes R4_RQ_LOCK -- all in interrupt context -- so any IF=1 task path
/// holding one of these must mask interrupts across the section, or a timer landing
/// mid-section would spin forever on a lock the interrupted task itself holds.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn lock_cli() -> u64 {
    let flags: u64;
    core::arch::asm!("pushfq; pop {}", out(reg) flags, options(nomem));
    core::arch::asm!("cli", options(nomem, nostack));
    flags
}
/// Restore the interrupt-enable state captured by lock_cli (re-enables ONLY if the
/// caller had interrupts enabled; leaves them masked otherwise -- so a section that
/// was already IF=0, e.g. the BSP PIT pump, stays masked).
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
pub(crate) unsafe fn unlock_sti(saved_flags: u64) {
    if saved_flags & 0x200 != 0 {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}

/// Slice 1 self-test: prove the unwired workload-distribution lock infrastructure is
/// sound before any path uses it -- each lock acquires (word -> 1) and releases (word
/// -> 0), the IRQ-save/restore pair round-trips, and each contention counter bumps on
/// a failed acquire. Runs single-threaded at boot (smp_init, IF=0, APs parked, locks
/// unwired) so storing a lock word directly to simulate a held lock is race-free.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn lock_infra_selftest() -> bool {
    // 1. Every lock starts free, acquires, and releases.
    if FS_LOCK.load(Ordering::Relaxed) != 0
        || STORAGE_LOCK.load(Ordering::Relaxed) != 0
        || NET_LOCK.load(Ordering::Relaxed) != 0
    {
        return false;
    }
    fs_lock();
    let fs_held = FS_LOCK.load(Ordering::Relaxed) == 1;
    fs_unlock();
    storage_lock();
    let st_held = STORAGE_LOCK.load(Ordering::Relaxed) == 1;
    storage_unlock();
    net_lock();
    let net_held = NET_LOCK.load(Ordering::Relaxed) == 1;
    net_unlock();
    if !(fs_held && st_held && net_held) {
        return false;
    }
    if FS_LOCK.load(Ordering::Relaxed) != 0
        || STORAGE_LOCK.load(Ordering::Relaxed) != 0
        || NET_LOCK.load(Ordering::Relaxed) != 0
    {
        return false;
    }
    // 2. IRQ-save/restore round-trips. At this boot point IF=0 (smp_init runs before
    // the first preemptible ring-3 entry), so the saved flags show IF clear and
    // unlock_sti must leave interrupts disabled (it only re-enables a caller that had
    // them on).
    let saved = lock_cli();
    let if_was_clear = saved & 0x200 == 0;
    unlock_sti(saved);
    let still_clear: u64;
    core::arch::asm!("pushfq; pop {}", out(reg) still_clear, options(nomem));
    if !if_was_clear || (still_clear & 0x200 != 0) {
        return false;
    }
    // 3. Each contention counter bumps by exactly one on a single failed acquire.
    // Simulate another CPU holding the lock (store 1 directly), confirm try_acquire
    // fails and the counter advances, then release. try_acquire is the production
    // inner step the spinning lock loops on, so this exercises the real bump path.
    let pairs: [(&AtomicU32, &AtomicU64, unsafe fn() -> bool); 3] = [
        (&FS_LOCK, &FS_CONTENTION, fs_try_acquire),
        (&STORAGE_LOCK, &STORAGE_CONTENTION, storage_try_acquire),
        (&NET_LOCK, &NET_CONTENTION, net_try_acquire),
    ];
    for (lock, counter, try_fn) in pairs.iter() {
        let before = counter.load(Ordering::Relaxed);
        lock.store(1, Ordering::Release); // held by a notional other CPU
        let got = try_fn();
        lock.store(0, Ordering::Release); // release the notional hold
        if got || counter.load(Ordering::Relaxed) != before + 1 {
            return false;
        }
    }
    true
}

// Ring-3 payload for an autonomously-scheduled task: bump its own yield_count via
// per-CPU current (sys_sysinfo op 16), then report (int 0x81) so the AP retires it
// and pulls the next. arg is 0, so the int-0x81 RDI is 1 (unused here).
//   48 89 FB              mov rbx, rdi
//   B8 3D 00 00 00        mov eax, 61      ; sys_sysinfo
//   BF 10 00 00 00        mov edi, 16      ; op 16 = bump my yield_count
//   31 F6 / 31 D2         xor esi/edx
//   CD 80                 int 0x80
//   48 89 DF              mov rdi, rbx
//   48 01 FF / 48 83 C7 01                 ; rdi = 2*arg+1
//   31 F6                 xor esi, esi
//   CD 81 / EB FE
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_PULL_CODE: [u8; 35] = [
    0x48, 0x89, 0xFB, 0xB8, 0x3D, 0x00, 0x00, 0x00, 0xBF, 0x10, 0x00, 0x00, 0x00, 0x31, 0xF6,
    0x31, 0xD2, 0xCD, 0x80, 0x48, 0x89, 0xDF, 0x48, 0x01, 0xFF, 0x48, 0x83, 0xC7, 0x01, 0x31,
    0xF6, 0xCD, 0x81, 0xEB, 0xFE,
];

/// Called from the AP park loop: when live-scheduler mode is on, claim one READY R4
/// task from the run set [SMP_LIVE_BASE, +COUNT) under R4_RQ_LOCK (atomic claim, so
/// the BSP/another CPU can't take the same one), set it as this CPU's per-CPU
/// current, and run it in ring 3. DIVERGES (never returns) if it claimed one -- it
/// enters ring 3 and resumes in ap_user_done, which retires the task and re-enters
/// the park loop to pull the next. Returns normally when off or nothing is ready.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn ap_pull_r4_task() {
    if SMP_LIVE_MODE.load(Ordering::Acquire) == 0 {
        return;
    }
    let slot: u64;
    core::arch::asm!("mov {}, gs:[0]", out(reg) slot, options(nostack));
    if slot == 0 || slot >= crate::arch_x86::MAX_TSS as u64 {
        return; // needs a per-CPU TSS for the ring-3 transition
    }
    // Affinity-based scheduling (full-os Part I.3): scan the WHOLE live task table
    // for an AP-eligible Ready task and claim it under the run-queue lock. The BSP's
    // r4_find_ready skips ap_eligible tasks, so this AP and the BSP work disjoint
    // sets -- the claim (Ready -> Running) is still locked so two APs can't take the
    // same one. (Replaces the earlier fixed [base,count) run-set scan; the live-sched
    // self-test now flags its tasks ap_eligible.)
    //
    // IF-safe claim (slice 4b): the AP's LAPIC-timer preemption handler takes
    // R4_RQ_LOCK to reschedule, so this -- the one R4_RQ_LOCK holder that runs with
    // IF=1 (the park loop) -- MUST disable interrupts across the claim, or a timer
    // landing mid-claim would spin forever on the lock we hold (self-deadlock).
    let saved_if: u64;
    core::arch::asm!("pushfq; pop {}", out(reg) saved_if, options(nomem));
    core::arch::asm!("cli", options(nomem, nostack));
    r4_rq_lock();
    let mut claimed = usize::MAX;
    let mut i = 1usize;
    while i < crate::R4_MAX_TASKS {
        if crate::R4_TASKS[i].ap_eligible && crate::R4_TASKS[i].state == crate::R4State::Ready {
            crate::R4_TASKS[i].state = crate::R4State::Running; // claim
            claimed = i;
            break;
        }
        i += 1;
    }
    r4_rq_unlock();
    if claimed == usize::MAX {
        // Nothing to run: restore the caller's interrupt state and return to the
        // park loop. (On the claimed paths below we never return -- the iretq into
        // ring 3 sets the task's own IF -- so restoring here is unnecessary there.)
        if saved_if & 0x200 != 0 {
            core::arch::asm!("sti", options(nomem, nostack));
        }
        return;
    }
    // Set up the claimed task's context for this AP and run it (autonomous dispatch:
    // the AP chose the task, the BSP did not). gen 0 -> no mailbox join. IF stays 0
    // through the setup (from the cli above) until the ring-3 entry restores it.
    let tid = claimed;
    // Preempt mode (slice 4b): resume the task with its FULL register context
    // (correct whether it is fresh OR was preempted mid-execution) and PREEMPTIBLE
    // (IF=1 from its saved RFLAGS). Diverges into ring 3.
    if SMP_PREEMPT_MODE.load(Ordering::Acquire) != 0 {
        ap_resume_r4_task(tid);
    }
    // Plain live-sched path (run-to-completion, IF=0 via enter_ring3_with_arg):
    // unchanged.
    AP_USER_CR3.store(crate::R4_TASKS[tid].pml4_phys, Ordering::Release);
    AP_USER_ENTRY.store(crate::R4_TASKS[tid].saved_frame[17], Ordering::Release);
    AP_USER_SP.store(crate::R4_TASKS[tid].saved_frame[20], Ordering::Release);
    AP_USER_TASKID.store(tid as u64, Ordering::Release);
    ap_run_user_task(0, 0);
}

/// Resume R4 task `tid` on THIS AP with its FULL saved register context, preemptibly
/// (live per-CPU scheduler slice 4b). The autonomous-pull counterpart of
/// `ap_run_user_task` that, unlike it, restores every register from saved_frame
/// rather than just RIP/RSP/RDI -- so a task pulled after being PREEMPTED resumes
/// exactly where it left off (its loop counter et al. survive). Records the per-CPU
/// current, saves the kernel CR3 for ap_user_trap to restore on the task's int 0x81,
/// switches to the task's address space, and iretq's into ring 3. DIVERGES.
///
/// Must be entered with IF=0 (the transient stack switch in iret_to_saved_frame must
/// not be interrupted); the restored RFLAGS sets the task's own IF=1.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn ap_resume_r4_task(tid: usize) -> ! {
    // This AP's per-CPU current task (gs:[16]); ap_user_trap reads it back to retire
    // the task when it reports via int 0x81.
    core::arch::asm!("mov qword ptr gs:[16], {v}", v = in(reg) tid as u64, options(nostack));
    let kcr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) kcr3, options(nomem, nostack));
    AP_SAVED_CR3.store(kcr3, Ordering::Release);
    AP_USER_GEN.store(0, Ordering::Release); // autonomous: no mailbox join
    crate::R4_TASKS[tid].state = crate::R4State::Running;
    crate::R4_TASKS[tid].dispatch_count += 1;
    let ucr3 = crate::R4_TASKS[tid].pml4_phys;
    core::arch::asm!("mov cr3, {}", in(reg) ucr3, options(nostack));
    crate::arch_x86::iret_to_saved_frame(core::ptr::addr_of!(crate::R4_TASKS[tid].saved_frame[0]));
}

/// BSP side: build a small run set of REAL R4 tasks (Ready, reserved slots, each its
/// own address space) and let an AP pull + run them autonomously, then verify each
/// ran exactly once. This is the live multi-CPU scheduler: APs draining the ready
/// set themselves under the run-queue lock, not the BSP hand-dispatching each task.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn smp_live_sched_selftest() -> bool {
    const K: usize = 3;
    let base: usize = crate::R4_MAX_TASKS - 1 - K; // below the migrate slot (MAX-1)
    const CODE_VA: u64 = 0x0140_0000;
    const STACK_TOP: u64 = 0x0013_0000;
    const STACK_PAGE: u64 = STACK_TOP - 0x1000;
    let kcr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) kcr3, options(nomem, nostack));
    let mut ases = [0u64; K];
    let mut made = 0usize;
    let mut j = 0usize;
    while j < K {
        let tid = base + j;
        let ucr3 = match crate::mm::address_space_create(kcr3) {
            Some(p) => p,
            None => break,
        };
        if !crate::mm::as_copyout(ucr3, CODE_VA, &AP_PULL_CODE)
            || !crate::mm::as_map_zeroed(ucr3, STACK_PAGE, 0x1000)
        {
            crate::mm::address_space_release(ucr3);
            break;
        }
        crate::r4_init_task(tid, CODE_VA, STACK_TOP, 0);
        crate::R4_TASKS[tid].pml4_phys = ucr3;
        crate::R4_TASKS[tid].yield_count = 0;
        crate::R4_TASKS[tid].ap_eligible = true; // affinity: only an AP claims it
        crate::R4_TASKS[tid].state = crate::R4State::Ready; // READY: an AP will claim it
        ases[j] = ucr3;
        made += 1;
        j += 1;
    }
    SMP_LIVE_RAN.store(0, Ordering::Release);
    SMP_LIVE_BASE.store(base as u64, Ordering::Release);
    SMP_LIVE_COUNT.store(made as u64, Ordering::Release);
    SMP_LIVE_MODE.store(1, Ordering::Release); // APs may now pull
    let mut spins = 0u64;
    while SMP_LIVE_RAN.load(Ordering::Acquire) < made as u64 && spins < 300_000_000 {
        core::hint::spin_loop();
        spins += 1;
    }
    SMP_LIVE_MODE.store(0, Ordering::Release); // stop pulling
    let ran = SMP_LIVE_RAN.load(Ordering::Acquire);
    // Teardown is gated on `ran == made` -- the live-test analogue of the sibling
    // tests' `result.is_some()` (see ap_r4_migrate_selftest:1358 / ap_r4_concurrent
    // _selftest:1436, and the UAF rationale at ap_user_selftest:724). Each retiring
    // task reaches ap_user_done only AFTER ap_user_trap restored the kernel CR3 on
    // that AP, and `ran` is the count of those completions read with Acquire, so
    // ran == made happens-after every AP left every task ucr3 -- it is provably off
    // all of them and idle. ONLY THEN may we free the page tables (else a claiming
    // AP still translating through ucr3 would run on freed, possibly-reused frames:
    // a cross-CPU UAF / PMM double-allocation) and write the slots. On a timeout
    // (ran < made: degraded boot / slow QEMU / an AP still in ring 3) we intentionally
    // LEAK the address spaces rather than free live page tables, exactly as the
    // siblings do. This boot self-test runs once; the timeout path is unreachable in
    // practice (the AP drains the run set within a few LAPIC-timer periods).
    let mut all_once = true;
    if ran == made as u64 {
        let mut z = 0usize;
        while z < made {
            let tid = base + z;
            // Retire under the run-queue lock: the same lock ap_user_done takes to
            // write R4_TASKS[done_tid].state, so these slot accesses never race a
            // late AP retirement (no unlocked data race on the shared task table).
            r4_rq_lock();
            if crate::R4_TASKS[tid].yield_count != 1 {
                all_once = false;
            }
            crate::R4_TASKS[tid].state = crate::R4State::Dead;
            r4_rq_unlock();
            crate::mm::address_space_release(ases[z]);
            z += 1;
        }
    } else {
        all_once = false; // could not verify: APs did not all quiesce
    }
    let ok = ran == made as u64 && made == K && all_once;
    serial_write(b"SMP: live sched ran=0x");
    serial_write_hex(ran);
    serial_write(b" ap-affinity");
    if ok {
        serial_write(b" ok\n");
    } else {
        serial_write(b" FAIL\n");
    }
    ok
}

/// CPU-bound ring-3 payload (live per-CPU scheduler slice 4): spin a bounded loop
/// -- long enough that this AP's periodic LAPIC timer fires (and preempts it)
/// several times -- then report via int 0x81 so the AP retires it. The loop
/// counter lives in RCX, which is saved/restored in the trap frame across each
/// preemption, so the task makes monotonic progress and always terminates.
///   48 B9 00 00 00 04 00 00 00 00   mov rcx, 0x04000000   ; ~67M iterations
///   48 FF C9                        dec rcx
///   75 FB                           jnz -5                ; back to `dec rcx`
///   31 FF                           xor edi, edi          ; report 0 (unused)
///   CD 81                           int 0x81              ; hand back to the kernel
///   EB FE                           jmp $                 ; (unreached; trampolined away)
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_SPIN_CODE: [u8; 21] = [
    0x48, 0xB9, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x48, 0xFF, 0xC9, 0x75, 0xFB, 0x31,
    0xFF, 0xCD, 0x81, 0xEB, 0xFE,
];

/// BSP side of slice 4 (preemptible AP tasks + the AP timer RESCHEDULES): create
/// TWO CPU-bound, AP-eligible R4 tasks, switch the AP into preempt mode, and let it
/// autonomously pull and run them PREEMPTIBLY (ring 3, IF=1). The AP's own periodic
/// LAPIC timer lands inside the running task (AP_PREEMPT_HITS) and the handler
/// RESCHEDULES -- saves the running task's full context and switches the core to the
/// OTHER task (AP_PREEMPT_SWITCHES), time-slicing the two on one application
/// processor with no BSP involvement. Each task's loop counter survives every
/// preemption (full save/restore), so BOTH run to completion (ran==2) -- a clean
/// completion that proves the save/restore round-trip is correct, while
/// switches>=2 proves the timer drove a real two-way context switch (A->B and B->A).
///
/// Slots base..base+K are distinct from migrate (MAX-1) and the live-sched run set
/// (MAX-2..MAX-4) and were retired to Dead before this runs.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn ap_preempt_selftest() -> bool {
    const K: usize = 2;
    let base: usize = crate::R4_MAX_TASKS - 6; // slots base..base+K, below live-sched's set
    const CODE_VA: u64 = 0x0140_0000; // exec-app window (loader clears NX)
    const STACK_TOP: u64 = 0x0013_0000;
    const STACK_PAGE: u64 = STACK_TOP - 0x1000;
    let kcr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) kcr3, options(nomem, nostack));
    let mut ases = [0u64; K];
    let mut made = 0usize;
    let mut j = 0usize;
    while j < K {
        let tid = base + j;
        let ucr3 = match crate::mm::address_space_create(kcr3) {
            Some(p) => p,
            None => break,
        };
        if !crate::mm::as_copyout(ucr3, CODE_VA, &AP_SPIN_CODE)
            || !crate::mm::as_map_zeroed(ucr3, STACK_PAGE, 0x1000)
        {
            crate::mm::address_space_release(ucr3);
            break;
        }
        crate::r4_init_task(tid, CODE_VA, STACK_TOP, 0);
        crate::R4_TASKS[tid].pml4_phys = ucr3;
        crate::R4_TASKS[tid].ap_eligible = true; // only an AP claims it
        crate::R4_TASKS[tid].state = crate::R4State::Ready; // READY: an AP will pull it
        ases[j] = ucr3;
        made += 1;
        j += 1;
    }
    AP_PREEMPT_HITS.store(0, Ordering::Release);
    AP_PREEMPT_SWITCHES.store(0, Ordering::Release);
    SMP_LIVE_RAN.store(0, Ordering::Release);
    SMP_PREEMPT_MODE.store(1, Ordering::Release); // launch IF=1 + count + reschedule
    SMP_LIVE_MODE.store(1, Ordering::Release); // APs may pull
    // Bounded wait: two CPU-bound spins, time-sliced, run far longer than the quick
    // live-sched tasks, so the budget is larger. The wait exits the instant the AP
    // retires both (SMP_LIVE_RAN reaches `made`); the cap is only a no-progress
    // backstop (never reached on success).
    let mut spins = 0u64;
    while SMP_LIVE_RAN.load(Ordering::Acquire) < made as u64 && spins < 8_000_000_000 {
        core::hint::spin_loop();
        spins += 1;
    }
    SMP_LIVE_MODE.store(0, Ordering::Release);
    SMP_PREEMPT_MODE.store(0, Ordering::Release);
    let ran = SMP_LIVE_RAN.load(Ordering::Acquire);
    let hits = AP_PREEMPT_HITS.load(Ordering::Acquire);
    let switches = AP_PREEMPT_SWITCHES.load(Ordering::Acquire);
    // Teardown gated on ran==made (both tasks completed -> each reached ap_user_done
    // only after ap_user_trap restored the kernel CR3 on the AP; `ran` is read with
    // Acquire, so the AP is provably off every task ucr3): only then free the page
    // tables + retire the slots under the run-queue lock. On a timeout, intentionally
    // leak rather than free a live page tree a claiming AP might still translate
    // through (a cross-CPU UAF) -- exactly as the sibling self-tests do.
    if ran == made as u64 {
        let mut z = 0usize;
        while z < made {
            r4_rq_lock();
            crate::R4_TASKS[base + z].state = crate::R4State::Dead;
            r4_rq_unlock();
            crate::mm::address_space_release(ases[z]);
            z += 1;
        }
    }
    let ok = made == K && ran == made as u64 && hits > 0 && switches >= 2;
    serial_write(b"SMP: ap preempt hits=0x");
    serial_write_hex(hits);
    serial_write(b" ran=0x");
    serial_write_hex(ran);
    serial_write(b" switches=0x");
    serial_write_hex(switches);
    serial_write(b"\n");
    if ok {
        serial_write(b"SMP: ap preempt ok\n");
    } else {
        serial_write(b"SMP: ap preempt FAIL\n");
    }
    ok
}

/// Affinity invariant proof for the BSP's live rotation (full-os Part I.3, live
/// per-CPU scheduler slice 3). The live scheduler reserves AP-eligible tasks for
/// application processors by having the BSP's `r4_find_ready` skip them. The
/// existing live-sched self-test parks its tasks in RESERVED slots ABOVE
/// R4_NUM_TASKS (which is 0 at smp_init), so the BSP's scan never even reaches
/// them and the `!ap_eligible` skip is never exercised in the rotation the BSP
/// actually runs. This test plants tasks INSIDE a (temporarily extended) live
/// window [1,R4_NUM_TASKS) and asserts, on the BSP, that:
///   (1) when a non-eligible task shares the window with eligible ones, the BSP
///       scheduler only ever returns the non-eligible one -- never an AP task;
///   (2) when EVERY task in the window is AP-eligible, the BSP has nothing to
///       run (the whole window is reserved for APs).
/// Together these prove the BSP and APs partition the SAME live task table into
/// disjoint sets, so no task is ever dispatched by two CPUs.
///
/// Safe to mutate R4_NUM_TASKS/R4_TASKS here: smp_init runs on the BSP with
/// IF=0 (no PIT preemption -- the PIC/PIT are not yet initialized) and
/// R4_NUM_TASKS==0, and SMP_LIVE_MODE is 0 (ap_pull_r4_task short-circuits, so no
/// AP scans the table). The window state is snapshotted and restored, so the
/// later go-lane boot (which sets R4_NUM_TASKS=1 and inits slot 0) is unaffected.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn r4_affinity_live_skip_selftest() -> bool {
    const A: usize = 1; // non-AP-eligible: the BSP MUST be able to pick this
    const B: usize = 2; // AP-eligible: the BSP MUST skip it
    const C: usize = 3; // AP-eligible: the BSP MUST skip it
    const WINDOW: usize = 4; // covers slots 0..=3; slot 0 stays Dead (EMPTY)
    if crate::R4_MAX_TASKS < WINDOW {
        return false;
    }
    // Snapshot what we touch (all Dead/EMPTY at smp_init, but be defensive so a
    // future reordering can't corrupt a live slot).
    let saved_num = crate::R4_NUM_TASKS;
    let (sa, ea, ca) = (
        crate::R4_TASKS[A].state,
        crate::R4_TASKS[A].ap_eligible,
        crate::R4_TASKS[A].sched_class,
    );
    let (sb, eb) = (crate::R4_TASKS[B].state, crate::R4_TASKS[B].ap_eligible);
    let (sc, ec) = (crate::R4_TASKS[C].state, crate::R4_TASKS[C].ap_eligible);

    crate::R4_NUM_TASKS = WINDOW;
    crate::R4_TASKS[A].state = crate::R4State::Ready;
    crate::R4_TASKS[A].ap_eligible = false;
    crate::R4_TASKS[A].sched_class = 0; // best-effort: the sole BSP candidate
    crate::R4_TASKS[B].state = crate::R4State::Ready;
    crate::R4_TASKS[B].ap_eligible = true;
    crate::R4_TASKS[C].state = crate::R4State::Ready;
    crate::R4_TASKS[C].ap_eligible = true;

    // (1) From every starting point the BSP scan only ever yields A (or nothing,
    //     when A itself is excluded) -- it must NEVER return an AP-eligible task.
    let mut bsp_disjoint = true;
    let mut x = 0usize;
    while x < WINDOW {
        if let Some(t) = crate::r4_find_ready(x) {
            if t != A {
                bsp_disjoint = false;
            }
        }
        x += 1;
    }
    // And the non-eligible task is genuinely runnable on the BSP: excluding an AP
    // task (or nothing) still finds A. This rules out a vacuous pass where the BSP
    // skipped EVERYTHING.
    let finds_a = matches!(crate::r4_find_ready(0), Some(t) if t == A)
        && matches!(crate::r4_find_ready(B), Some(t) if t == A)
        && matches!(crate::r4_find_ready(C), Some(t) if t == A);

    // (2) Make the whole window AP-eligible: the BSP now has nothing to run.
    crate::R4_TASKS[A].ap_eligible = true;
    let mut bsp_starved = true;
    let mut y = 0usize;
    while y < WINDOW {
        if crate::r4_find_ready(y).is_some() {
            bsp_starved = false;
        }
        y += 1;
    }

    // Restore the window and the live count exactly as we found them.
    crate::R4_TASKS[A].state = sa;
    crate::R4_TASKS[A].ap_eligible = ea;
    crate::R4_TASKS[A].sched_class = ca;
    crate::R4_TASKS[B].state = sb;
    crate::R4_TASKS[B].ap_eligible = eb;
    crate::R4_TASKS[C].state = sc;
    crate::R4_TASKS[C].ap_eligible = ec;
    crate::R4_NUM_TASKS = saved_num;

    let ok = bsp_disjoint && finds_a && bsp_starved;
    serial_write(b"SMP: affinity live-skip ");
    if ok {
        serial_write(b"ok\n");
    } else {
        serial_write(b"FAIL\n");
    }
    ok
}

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
        // Claim a per-CPU slot and point this AP's GS base at it BEFORE arming
        // the LAPIC timer, so the timer ISR's `gs:` access always has a base.
        // If the slot overflows MAX_CPUS (more CPUs than slots) set NEITHER the
        // GS base NOR the timer: otherwise the timer ISR would run `gs:[8]` with
        // a zero GS base and fault. A surplus AP still checks in and parks (its
        // IPI + spurious vectors work); it simply has no per-CPU preemption clock.
        let slot = PERCPU_NEXT.fetch_add(1, Ordering::SeqCst) as usize;
        if slot < MAX_CPUS {
            percpu_init(slot);
            lapic_timer_start(); // this AP's own periodic preemption clock
            // SMP capstone: give this AP its own TSS (rsp0 = a per-CPU kernel
            // stack) so it can take a ring-3->ring-0 transition and run user
            // tasks. Bounded by MAX_TSS; a surplus AP simply cannot run ring 3.
            // Done before the check-in below, so once the BSP sees every AP
            // online they are all ready to be handed a user task.
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            {
                if slot < crate::arch_x86::MAX_TSS {
                    crate::arch_x86::tss_init_cpu(slot, crate::arch_x86::ap_kstack_top(slot));
                }
            }
        }
        APS_ONLINE.fetch_add(1, Ordering::SeqCst);
        loop {
            // Run any work the BSP dispatched (no-op when none pending) + drain
            // this CPU's own run queue + (in live-scheduler mode) autonomously claim
            // and run a ready R4 task, then sleep until the next interrupt (the
            // periodic LAPIC timer wakes us to poll again). This is the AP doing
            // real kernel work, not parking.
            ap_poll_work();
            ap_poll_rq();
            // The live-scheduler pull only exists in the go_test lane; ap_entry
            // itself is built for every lane (incl. the base os.iso), so gate it.
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            {
                ap_pull_r4_task();
            }
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
            BSP_LAPIC_ID.store(bsp, Ordering::Release); // for is_bsp() in the syscall path
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
            // Per-CPU storage via GS: each online AP recorded its slot index
            // THROUGH its own GS base; verify each slot holds the right index.
            // ONLY on the success path (every AP checked in): an AP writes
            // cpu_index before its SeqCst check-in, so once all are in the read
            // is ordered after every write (race-free) and slots 1..=online are
            // exactly the claimed, populated set. On the bounded-spin timeout
            // path an AP may hold a slot without having checked in (slot-claim
            // order is independent of check-in order) and could be mid-write, so
            // the read would race — skip it and report a timeout instead.
            if online >= expected {
                let mut percpu_ok = online > 0;
                let mut s = 1u64;
                while s <= online && (s as usize) < MAX_CPUS {
                    let idx =
                        core::ptr::read_volatile(core::ptr::addr_of!(PERCPU[s as usize].cpu_index));
                    if idx != s {
                        percpu_ok = false;
                    }
                    s += 1;
                }
                serial_write(b"SMP: percpu ");
                if percpu_ok {
                    serial_write(b"ok\n");
                } else {
                    serial_write(b"FAIL\n");
                }
            } else {
                serial_write(b"SMP: percpu timeout FAIL\n");
            }
            // Cross-CPU work dispatch: hand a real computation (sum 1..=1000) to
            // an AP, which claims it and runs it ON ITS OWN CORE, then reports the
            // result. Proof the APs execute dispatched kernel work — the per-CPU
            // execution primitive a scheduler runs tasks on.
            let work_ok = match smp_dispatch_work(1, 1000) {
                Some(r) => r == 500_500,
                None => false,
            };
            serial_write(b"SMP: ap work ");
            if work_ok {
                serial_write(b"ok\n");
            } else {
                serial_write(b"FAIL\n");
            }
            // Per-CPU run queues: give every AP its own queue and confirm each
            // drained it concurrently with the right per-CPU total — the
            // scheduler data structure a real SMP scheduler dispatches onto.
            // Gated on a fully-online boot (like the lock/percpu self-tests):
            // only then are slots 1..=online provably the checked-in,
            // GS-initialized set this enqueues to. On the bounded-spin timeout
            // path a claimed slot may have no GS base yet, so skip rather than
            // stall on a queue no AP will drain.
            if online >= expected {
                serial_write(b"SMP: runqueue ");
                if ap_runqueue_selftest() {
                    serial_write(b"ok\n");
                } else {
                    serial_write(b"FAIL\n");
                }
            } else {
                serial_write(b"SMP: runqueue timeout FAIL\n");
            }
            // Per-CPU affinity + load distribution: route DISTINCT work to each
            // core and confirm each ran only its own (the basis for balancing).
            if online >= expected {
                serial_write(b"SMP: affinity ");
                if ap_affinity_selftest() {
                    serial_write(b"ok\n");
                } else {
                    serial_write(b"FAIL\n");
                }
            }
            // Kernel-wide locking (slice 5): the BSP and an AP allocate physical
            // frames CONCURRENTLY and the BSP proves the batches are disjoint -- the
            // PMM lock serializing the bitmap RMW across cores. (Uses the async work
            // dispatch, which is go-lane only, so this test is gated to the go lane;
            // the PMM lock itself is compiled into every lane.)
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            if online >= expected {
                serial_write(b"SMP: pmm smp ");
                if pmm_smp_selftest() {
                    serial_write(b"ok\n");
                } else {
                    serial_write(b"FAIL\n");
                }
                // Kernel heap (slice 5b): BSP + AP allocate heap blocks concurrently;
                // the BSP proves no block was double-handed-out / overlapped (each
                // still holds its own CPU's stamped pattern) -- the heap lock
                // serializing the free-list across cores.
                serial_write(b"SMP: heap smp ");
                if heap_smp_selftest() {
                    serial_write(b"ok\n");
                } else {
                    serial_write(b"FAIL\n");
                }
            }
            // Capstone: run a real ring-3 USER task on an application processor.
            // The BSP builds the user address space + dispatches; an AP enters
            // ring 3, runs it on its own core, and reports back.
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            {
                // Workload-distribution lock infrastructure (slice 1): the coarse FS/
                // STORAGE/NET leaf locks + the IRQ-save/restore pair + contention
                // counters are sound (acquire/release toggles, counters bump) before
                // any path wires them. Unwired here; slices 2-5 guard real FS/net/
                // block state with them.
                serial_write(b"SMP-LOCKS: ");
                if lock_infra_selftest() {
                    serial_write(b"fs/storage/net init ok\n");
                } else {
                    serial_write(b"FAIL\n");
                }
                // Affinity invariant: the BSP's scheduler skips AP-eligible tasks
                // INSIDE its own live rotation (SMP_LIVE_MODE is still 0 here, so no
                // AP touches the table while we probe r4_find_ready).
                let _ = r4_affinity_live_skip_selftest();
                let user_ok = ap_user_selftest();
                serial_write(b"SMP: ap user task ");
                if user_ok {
                    serial_write(b"ok\n");
                } else {
                    serial_write(b"FAIL\n");
                }
                // Migrate a REAL R4 task (a scheduler task struct) to an AP.
                let _ = ap_r4_migrate_selftest();
                // Prove two tasks run on two CPUs AT ONCE (BSP + a ring-3 AP task).
                let _ = ap_r4_concurrent_selftest();
                // Live SMP scheduler: an AP autonomously pulls READY R4 tasks from
                // the run set (claiming each under the run-queue lock) and runs them.
                let _ = smp_live_sched_selftest();
                // Live SMP scheduler slice 4: an AP runs two CPU-bound tasks
                // PREEMPTIBLY (IF=1); its own LAPIC timer fires inside the running
                // ring-3 task and RESCHEDULES the core between the two (time-slicing).
                let _ = ap_preempt_selftest();
            }
        }
    }
}

/// SMP scheduler (full-os guide Part I.3): migrate a REAL R4 task to an
/// application processor. Unlike the capstone (which runs a free-floating ring-3
/// payload), this sets up an actual R4_TASKS scheduler entry via r4_init_task —
/// with its own address space (pml4_phys) and ring-3 entry context (saved_frame)
/// — and runs THAT task's context on the AP, tracking its real tid as the AP's
/// per-CPU `current` and servicing its syscalls on the AP's own core.
///
/// The task uses a reserved slot (R4_MAX_TASKS-1), created in state Running and
/// beyond R4_NUM_TASKS, so the BSP's scheduler (r4_find_ready takes only Ready
/// tasks within R4_NUM_TASKS) never races it. r4_tasks_init() has already grown
/// the task table (it runs before smp_init), so the slot is valid here.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn ap_r4_migrate_selftest() -> bool {
    const CODE_VA: u64 = 0x0140_0000; // exec-app window (NX cleared by the loader)
    const STACK_TOP: u64 = 0x0013_0000;
    const STACK_PAGE: u64 = STACK_TOP - 0x1000;
    const ARG: u64 = 33;
    let mig_tid: usize = crate::R4_MAX_TASKS - 1;
    let kcr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) kcr3, options(nomem, nostack));
    let ucr3 = match crate::mm::address_space_create(kcr3) {
        Some(p) => p,
        None => return false,
    };
    if !crate::mm::as_copyout(ucr3, CODE_VA, &AP_R4_CODE)
        || !crate::mm::as_map_zeroed(ucr3, STACK_PAGE, 0x1000)
    {
        crate::mm::address_space_release(ucr3);
        return false;
    }
    // Register a real R4 task for it. State Running keeps the BSP scheduler off it.
    crate::r4_init_task(mig_tid, CODE_VA, STACK_TOP, 0);
    crate::R4_TASKS[mig_tid].pml4_phys = ucr3;
    crate::R4_TASKS[mig_tid].state = crate::R4State::Running;
    // A sentinel uid so the task's getuid (which reads R4_TASKS[per-cpu current].uid
    // on the AP) returns a value provably tied to THIS migrated slot; yield_count
    // starts at 0 so the task's op-16 WRITE (bump via per-CPU current) lands here.
    crate::R4_TASKS[mig_tid].uid = 0x77;
    crate::R4_TASKS[mig_tid].yield_count = 0;
    // Migrate its CR3 + ring-3 entry context (RIP/RSP from saved_frame) to the AP,
    // and publish its real tid as the per-CPU `current` the AP records.
    AP_USER_CR3.store(crate::R4_TASKS[mig_tid].pml4_phys, Ordering::Release);
    AP_USER_ENTRY.store(crate::R4_TASKS[mig_tid].saved_frame[17], Ordering::Release);
    AP_USER_SP.store(crate::R4_TASKS[mig_tid].saved_frame[20], Ordering::Release);
    AP_USER_TASKID.store(mig_tid as u64, Ordering::Release);
    AP_USER_CURRENT.store(0, Ordering::Release);
    AP_USER_SYSRET.store(0, Ordering::Release);
    let result = smp_dispatch_work(2, ARG);
    let cpu = AP_USER_CPU.load(Ordering::Acquire);
    let cur = AP_USER_CURRENT.load(Ordering::Acquire);
    // scuid: the uid the task's OWN real syscall (getuid -> R4_TASKS[r4_current_smp()]
    // .uid) resolved while running on the AP. The task reported it in RSI ->
    // AP_USER_SYSRET. Equal to the sentinel iff per-CPU current indexed this slot.
    let scuid = AP_USER_SYSRET.load(Ordering::Acquire);
    // The WRITE the task made through per-CPU current (op 16 bumped this slot's
    // yield_count from 0). Read after the join's Acquire so the AP's write is visible.
    let scyc = crate::R4_TASKS[mig_tid].yield_count;
    crate::R4_TASKS[mig_tid].state = crate::R4State::Dead; // retire the migrated task
    if result.is_some() {
        crate::mm::address_space_release(ucr3);
    }
    // Proof: the task ran in ring 3 on an AP (result, cpu), its per-CPU `current`
    // round-tripped via GS (cur == real tid), and TWO real syscalls it executed on
    // the AP resolved its OWN slot through per-CPU current -- a READ (getuid ->
    // scuid == the sentinel uid) and a WRITE (op 16 -> scyc, this slot's yield_count
    // now 1). The per-CPU R4_CURRENT reroute working end to end, both directions,
    // indexing the real task table, for a real R4 task.
    let ok = matches!(result, Some(v) if v == ARG * 2 + 1)
        && cpu >= 1
        && cur == mig_tid as u64
        && scuid == 0x77
        && scyc == 1;
    serial_write(b"SMP: ap r4 migrate tid=0x");
    serial_write_hex(mig_tid as u64);
    serial_write(b" cur=0x");
    serial_write_hex(cur);
    serial_write(b" scuid=0x");
    serial_write_hex(scuid);
    serial_write(b" scyc=0x");
    serial_write_hex(scyc);
    if ok {
        serial_write(b" ok\n");
    } else {
        serial_write(b" FAIL\n");
    }
    ok
}

/// SMP concurrency proof (full-os guide Part I.3): a ring-3 task on an application
/// processor and the BSP executing AT THE SAME TIME. The BSP dispatches the task
/// ASYNCHRONOUSLY (`smp_dispatch_async` — it does NOT block), then performs a
/// rendezvous with it: the task (via `sys_sysinfo` op 15 -> `smp_rendezvous_ap`)
/// signals arrival and waits in-kernel on the AP for the BSP's acknowledgement; the
/// BSP waits for the arrival, acks, then joins. The handshake can only close if both
/// CPUs are live at the same instant — were the BSP blocked (as with the synchronous
/// dispatch), the AP's bounded wait would time out — so a completed rendezvous
/// (the task returns 0xAC) is proof of genuine multi-CPU multitasking.
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
unsafe fn ap_r4_concurrent_selftest() -> bool {
    const CODE_VA: u64 = 0x0140_0000;
    const STACK_TOP: u64 = 0x0013_0000;
    const STACK_PAGE: u64 = STACK_TOP - 0x1000;
    const ARG: u64 = 44;
    const TASK_ID: u64 = 0x5B;
    let kcr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) kcr3, options(nomem, nostack));
    let ucr3 = match crate::mm::address_space_create(kcr3) {
        Some(p) => p,
        None => return false,
    };
    if !crate::mm::as_copyout(ucr3, CODE_VA, &AP_RV_CODE)
        || !crate::mm::as_map_zeroed(ucr3, STACK_PAGE, 0x1000)
    {
        crate::mm::address_space_release(ucr3);
        return false;
    }
    AP_USER_CR3.store(ucr3, Ordering::Release);
    AP_USER_ENTRY.store(CODE_VA, Ordering::Release);
    AP_USER_SP.store(STACK_TOP, Ordering::Release);
    AP_USER_TASKID.store(TASK_ID, Ordering::Release);
    AP_USER_CURRENT.store(0, Ordering::Release);
    AP_USER_SYSRET.store(0, Ordering::Release);
    SMP_RV.store(0, Ordering::Release);
    // Dispatch WITHOUT blocking, then rendezvous concurrently with the AP task.
    let gen = smp_dispatch_async(2, ARG);
    let mut spins = 0u64;
    while SMP_RV.load(Ordering::Acquire) < 1 && spins < 200_000_000 {
        core::hint::spin_loop();
        spins += 1;
    }
    let arrived = SMP_RV.load(Ordering::Acquire) >= 1;
    if arrived {
        SMP_RV.store(2, Ordering::Release); // BSP acknowledges -> releases the AP task
    }
    let result = smp_join(gen);
    let rv = AP_USER_SYSRET.load(Ordering::Acquire); // 0xAC if the task saw the ack
    if result.is_some() {
        crate::mm::address_space_release(ucr3);
    }
    let ok = matches!(result, Some(v) if v == ARG * 2 + 1) && arrived && rv == 0xAC;
    serial_write(b"SMP: ap+bsp concurrent rv=0x");
    serial_write_hex(rv);
    if ok {
        serial_write(b" ok\n");
    } else {
        serial_write(b" FAIL\n");
    }
    ok
}
