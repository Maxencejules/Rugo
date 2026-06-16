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

/// Run a dispatched work item on the current CPU. Kind 1 = sum 1..=arg, computed
/// iteratively (a real workload the dispatcher can independently check).
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

// Ring-3 payload for the REAL-R4-task migration test. It issues a genuine syscall
// that resolves "which task am I" from PER-CPU state (sys_sysinfo op 14 ->
// r4_current_smp), then reports the kernel-resolved tid so the BSP can confirm a
// syscall ON THE AP saw the migrated task as its current — the per-CPU R4_CURRENT
// mechanism working through the real syscall path (not just a side variable):
//   48 89 FB              mov rbx, rdi    ; save arg
//   B8 3D 00 00 00        mov eax, 61     ; nr 61 = sys_sysinfo
//   BF 0E 00 00 00        mov edi, 14     ; op 14 = SMP per-CPU current tid
//   31 F6                 xor esi, esi    ; a2 = 0
//   31 D2                 xor edx, edx    ; a3 = 0
//   CD 80                 int 0x80        ; rax = resolved current tid
//   48 89 C6              mov rsi, rax    ; report resolved tid in RSI
//   48 89 DF              mov rdi, rbx    ; restore arg
//   48 01 FF              add rdi, rdi    ; rdi = 2*arg
//   48 83 C7 01           add rdi, 1      ; rdi = 2*arg+1 (the ran-in-ring3 proof)
//   CD 81                 int 0x81        ; report to ap_user_trap
//   EB FE               1: jmp 1b
#[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
static AP_R4_CODE: [u8; 36] = [
    0x48, 0x89, 0xFB, 0xB8, 0x3D, 0x00, 0x00, 0x00, 0xBF, 0x0E, 0x00, 0x00, 0x00, 0x31, 0xF6,
    0x31, 0xD2, 0xCD, 0x80, 0x48, 0x89, 0xC6, 0x48, 0x89, 0xDF, 0x48, 0x01, 0xFF, 0x48, 0x83,
    0xC7, 0x01, 0xCD, 0x81, 0xEB, 0xFE,
];

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
    core::arch::asm!("mov cr3, {}", in(reg) ucr3, options(nostack));
    crate::arch_x86::enter_ring3_with_arg(entry, usp, arg);
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
        loop {
            ap_poll_work();
            ap_poll_rq();
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

/// LAPIC-timer service routine (from trap_handler vector 241): tick + EOI.
pub(crate) unsafe fn lapic_timer_handler() {
    AP_TICKS.fetch_add(1, Ordering::SeqCst);
    // Bump THIS CPU's per-CPU tick counter through the GS base: no lock and no
    // CPU-id lookup, because each CPU's GS base points at its own slot. This is
    // the exact access pattern a per-CPU scheduler uses for `current`/run-queue.
    core::arch::asm!("add qword ptr gs:[8], 1", options(nostack));
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
            // this CPU's own run queue, then sleep until the next interrupt (the
            // periodic LAPIC timer wakes us to poll again). This is the AP doing
            // real kernel work, not parking.
            ap_poll_work();
            ap_poll_rq();
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
            // Capstone: run a real ring-3 USER task on an application processor.
            // The BSP builds the user address space + dispatches; an AP enters
            // ring 3, runs it on its own core, and reports back.
            #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
            {
                let user_ok = ap_user_selftest();
                serial_write(b"SMP: ap user task ");
                if user_ok {
                    serial_write(b"ok\n");
                } else {
                    serial_write(b"FAIL\n");
                }
                // Migrate a REAL R4 task (a scheduler task struct) to an AP.
                let _ = ap_r4_migrate_selftest();
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
    // sctid: the tid the task's OWN syscall (sys_sysinfo op 14 -> r4_current_smp)
    // resolved while running on the AP. The task reported it in RSI -> AP_USER_SYSRET.
    let sctid = AP_USER_SYSRET.load(Ordering::Acquire);
    crate::R4_TASKS[mig_tid].state = crate::R4State::Dead; // retire the migrated task
    if result.is_some() {
        crate::mm::address_space_release(ucr3);
    }
    // Proof: the task ran in ring 3 on an AP (result, cpu), its per-CPU `current`
    // round-tripped via GS (cur), AND a real syscall it executed on the AP resolved
    // that same real tid from per-CPU state (sctid) -- the per-CPU R4_CURRENT
    // mechanism working end-to-end through the syscall path for a real R4 task.
    let ok = matches!(result, Some(v) if v == ARG * 2 + 1)
        && cpu >= 1
        && cur == mig_tid as u64
        && sctid == mig_tid as u64;
    serial_write(b"SMP: ap r4 migrate tid=0x");
    serial_write_hex(mig_tid as u64);
    serial_write(b" cur=0x");
    serial_write_hex(cur);
    serial_write(b" sctid=0x");
    serial_write_hex(sctid);
    if ok {
        serial_write(b" ok\n");
    } else {
        serial_write(b" FAIL\n");
    }
    ok
}
