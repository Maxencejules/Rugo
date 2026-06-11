// Timer interrupt handling and the cooperative scheduler test harness.

use crate::{outb, qemu_exit, serial_write};

#[cfg(any(feature = "sched_test", feature = "go_test"))]
const PIC1_CMD: u16 = 0x20;
#[cfg(any(feature = "sched_test", feature = "go_test"))]
const PIC1_DATA: u16 = 0x21;
#[cfg(any(feature = "sched_test", feature = "go_test"))]
const PIC2_CMD: u16 = 0xA0;
#[cfg(any(feature = "sched_test", feature = "go_test"))]
const PIC2_DATA: u16 = 0xA1;

/// Remap the 8259A PICs to vectors 32..47 and mask every line but IRQ0
/// (the PIT). Shared by the sched_test lane and default-lane preemption.
#[cfg(any(feature = "sched_test", feature = "go_test"))]
pub(crate) unsafe fn pic_init() {
    outb(PIC1_CMD, 0x11);
    outb(PIC2_CMD, 0x11);
    outb(PIC1_DATA, 32);
    outb(PIC2_DATA, 40);
    outb(PIC1_DATA, 0x04);
    outb(PIC2_DATA, 0x02);
    outb(PIC1_DATA, 0x01);
    outb(PIC2_DATA, 0x01);
    outb(PIC1_DATA, 0xFE);
    outb(PIC2_DATA, 0xFF);
}

#[cfg(any(feature = "sched_test", feature = "go_test"))]
pub(crate) unsafe fn pic_send_eoi(irq: u8) {
    if irq >= 8 {
        outb(PIC2_CMD, 0x20);
    }
    outb(PIC1_CMD, 0x20);
}

/// Unmask one master-PIC line (e.g. IRQ1 for the PS/2 keyboard).
#[cfg(any(feature = "sched_test", feature = "go_test"))]
pub(crate) unsafe fn pic_unmask(irq: u8) {
    if irq < 8 {
        let mask = crate::arch_x86::inb(PIC1_DATA);
        outb(PIC1_DATA, mask & !(1 << irq));
    }
}

#[cfg(any(feature = "sched_test", feature = "go_test"))]
pub(crate) unsafe fn pit_init(freq: u32) {
    let divisor = 1_193_182u32 / freq;
    outb(0x43, 0x34);
    outb(0x40, (divisor & 0xFF) as u8);
    outb(0x40, ((divisor >> 8) & 0xFF) as u8);
}

#[cfg(feature = "sched_test")]
static mut TICK_COUNT: u64 = 0;

#[cfg(feature = "sched_test")]
const MAX_THREADS: usize = 4;
#[cfg(feature = "sched_test")]
const THREAD_STACK_SIZE: usize = 16384;

#[cfg(feature = "sched_test")]
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum ThreadState {
    Dead = 0,
    Ready = 1,
    Running = 2,
}

#[cfg(feature = "sched_test")]
#[derive(Clone, Copy)]
#[repr(C)]
struct Thread {
    rsp: u64,
    state: ThreadState,
}

#[cfg(feature = "sched_test")]
impl Thread {
    const EMPTY: Self = Self {
        rsp: 0,
        state: ThreadState::Dead,
    };
}

#[cfg(feature = "sched_test")]
static mut THREADS: [Thread; MAX_THREADS] = [Thread::EMPTY; MAX_THREADS];
#[cfg(feature = "sched_test")]
static mut THREAD_STACKS: [[u8; THREAD_STACK_SIZE]; MAX_THREADS] =
    [[0u8; THREAD_STACK_SIZE]; MAX_THREADS];
#[cfg(feature = "sched_test")]
static mut CURRENT_THREAD: usize = 0;
#[cfg(feature = "sched_test")]
static mut NUM_THREADS: usize = 0;

#[cfg(feature = "sched_test")]
extern "C" {
    fn context_switch(old_rsp: *mut u64, new_rsp: u64);
    fn thread_entry_trampoline();
}

#[cfg(feature = "sched_test")]
pub(crate) unsafe fn sched_init() {
    THREADS[0].state = ThreadState::Running;
    THREADS[0].rsp = 0;
    CURRENT_THREAD = 0;
    NUM_THREADS = 1;
}

#[cfg(feature = "sched_test")]
pub(crate) unsafe fn thread_create(func: extern "C" fn()) {
    let tid = NUM_THREADS;
    NUM_THREADS += 1;
    let sp_top = THREAD_STACKS[tid].as_mut_ptr().add(THREAD_STACK_SIZE) as u64;
    let mut sp = sp_top;
    sp -= 8;
    *(sp as *mut u64) = thread_exit as *const () as u64;
    sp -= 8;
    *(sp as *mut u64) = func as *const () as u64;
    sp -= 8;
    *(sp as *mut u64) = thread_entry_trampoline as *const () as u64;
    sp -= 8;
    *(sp as *mut u64) = 0;
    sp -= 8;
    *(sp as *mut u64) = 0;
    sp -= 8;
    *(sp as *mut u64) = 0;
    sp -= 8;
    *(sp as *mut u64) = 0;
    sp -= 8;
    *(sp as *mut u64) = 0;
    sp -= 8;
    *(sp as *mut u64) = 0;
    THREADS[tid].rsp = sp;
    THREADS[tid].state = ThreadState::Ready;
}

#[cfg(feature = "sched_test")]
unsafe fn schedule() {
    let cur = CURRENT_THREAD;
    let mut next = (cur + 1) % NUM_THREADS;
    let start = next;
    loop {
        if THREADS[next].state == ThreadState::Ready {
            break;
        }
        next = (next + 1) % NUM_THREADS;
        if next == start {
            return;
        }
    }
    if next == cur {
        return;
    }
    if THREADS[cur].state == ThreadState::Running {
        THREADS[cur].state = ThreadState::Ready;
    }
    THREADS[next].state = ThreadState::Running;
    CURRENT_THREAD = next;
    let old_rsp = &mut THREADS[cur].rsp as *mut u64;
    let new_rsp = THREADS[next].rsp;
    context_switch(old_rsp, new_rsp);
}

#[cfg(feature = "sched_test")]
pub(crate) unsafe fn handle_timer_irq() {
    TICK_COUNT += 1;
    if TICK_COUNT == 100 {
        serial_write(b"TICK: 100\n");
    }
    pic_send_eoi(0);
    if TICK_COUNT >= 400 {
        qemu_exit(0x31);
        loop { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
    }
    schedule();
}

#[cfg(feature = "sched_test")]
pub(crate) unsafe fn yield_now() {
    if NUM_THREADS > 0 {
        schedule();
    }
}

#[cfg(feature = "sched_test")]
extern "C" fn thread_exit() {
    unsafe {
        THREADS[CURRENT_THREAD].state = ThreadState::Dead;
        schedule();
    }
    loop {
        unsafe { core::arch::asm!("cli; hlt", options(nomem, nostack)); }
    }
}
