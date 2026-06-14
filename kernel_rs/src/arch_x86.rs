// x86-64 bring-up and low-level CPU entry helpers.

#[inline(always)]
pub(crate) unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack));
}

#[inline(always)]
pub(crate) unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack));
    val
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
#[inline(always)]
pub(crate) unsafe fn outw(port: u16, value: u16) {
    core::arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack));
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
#[inline(always)]
pub(crate) unsafe fn outl(port: u16, value: u32) {
    core::arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack));
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
#[inline(always)]
pub(crate) unsafe fn inl(port: u16) -> u32 {
    let val: u32;
    core::arch::asm!("in eax, dx", out("eax") val, in("dx") port, options(nomem, nostack));
    val
}

#[cfg(any(feature = "blk_test", feature = "blk_invariants_test", feature = "fs_test", feature = "net_test", feature = "go_test"))]
#[inline(always)]
pub(crate) unsafe fn inw(port: u16) -> u16 {
    let val: u16;
    core::arch::asm!("in ax, dx", out("ax") val, in("dx") port, options(nomem, nostack));
    val
}

const DEBUG_EXIT_PORT: u16 = 0xF4;
const COM1_LSR: u16 = 0x3FD;

pub(crate) fn qemu_exit(code: u8) {
    unsafe {
        // Drain the UART transmitter and give the host a moment to pull
        // the last serial bytes off the wire: an immediate debug-exit
        // tears the chardev socket down with data still in flight and
        // the capture side loses the tail of the boot transcript.
        let mut spins = 0u32;
        while inb(COM1_LSR) & 0x40 == 0 && spins < 100_000 {
            spins += 1;
        }
        let mut settle = 0u32;
        while settle < 20_000 {
            outb(0x80, 0);
            settle += 1;
        }
        outb(DEBUG_EXIT_PORT, code);
    }
}

#[repr(C, packed)]
struct DtPtr {
    limit: u16,
    base: u64,
}

static mut GDT: [u64; 7] = [
    0x0000_0000_0000_0000,
    0x00AF_9A00_0000_FFFF,
    0x00CF_9200_0000_FFFF,
    0x00CF_F200_0000_FFFF,
    0x00AF_FA00_0000_FFFF,
    0,
    0,
];

pub(crate) unsafe fn gdt_init() {
    let limit = (core::mem::size_of_val(&GDT) - 1) as u16;
    let base = GDT.as_ptr() as u64;
    let ptr = DtPtr { limit, base };
    core::arch::asm!("lgdt [{}]", in(reg) &ptr);
    core::arch::asm!(
        "push 0x08",
        "lea {tmp}, [rip + 2f]",
        "push {tmp}",
        ".byte 0x48, 0xCB",
        "2:",
        "mov {tmp:x}, 0x10",
        "mov ds, {tmp:x}",
        "mov es, {tmp:x}",
        "mov fs, {tmp:x}",
        "mov gs, {tmp:x}",
        "mov ss, {tmp:x}",
        tmp = lateout(reg) _,
    );
}

cfg_user! {
    #[repr(C, packed)]
    struct Tss {
        reserved0: u32,
        rsp0: u64,
        rsp1: u64,
        rsp2: u64,
        reserved1: u64,
        ist: [u64; 7],
        reserved2: u64,
        reserved3: u16,
        iopb_offset: u16,
    }

    static mut TSS: Tss = Tss {
        reserved0: 0,
        rsp0: 0, rsp1: 0, rsp2: 0,
        reserved1: 0,
        ist: [0; 7],
        reserved2: 0,
        reserved3: 0,
        iopb_offset: 104,
    };

    pub(crate) unsafe fn tss_init(kernel_stack_top: u64) {
        TSS.rsp0 = kernel_stack_top;
        let tss_addr = &TSS as *const Tss as u64;
        GDT[5] = (103u64)
                | ((tss_addr & 0xFFFF) << 16)
                | (((tss_addr >> 16) & 0xFF) << 32)
                | (0x89u64 << 40)
                | (((tss_addr >> 24) & 0xFF) << 56);
        GDT[6] = tss_addr >> 32;
        let limit = (core::mem::size_of_val(&GDT) - 1) as u16;
        let base = GDT.as_ptr() as u64;
        let gdt_ptr = DtPtr { limit, base };
        core::arch::asm!("lgdt [{}]", in(reg) &gdt_ptr);
        core::arch::asm!(
            "mov ax, 0x28",
            "ltr ax",
            out("ax") _,
            options(nostack),
        );
    }

    pub(crate) unsafe fn enter_ring3_at(code_va: u64, user_sp: u64) -> ! {
        core::arch::asm!(
            "push 0x1B",
            "push {stack}",
            "push 0x002",
            "push 0x23",
            "push {code}",
            "iretq",
            stack = in(reg) user_sp,
            code = in(reg) code_va,
            options(noreturn),
        );
    }

    /// Like enter_ring3_at but with RFLAGS.IF set: the task can be
    /// preempted by the PIT. Only safe once the PIC is remapped + masked.
    #[cfg(feature = "go_test")]
    pub(crate) unsafe fn enter_ring3_preemptible(code_va: u64, user_sp: u64) -> ! {
        core::arch::asm!(
            "push 0x1B",
            "push {stack}",
            "push 0x202",
            "push 0x23",
            "push {code}",
            "iretq",
            stack = in(reg) user_sp,
            code = in(reg) code_va,
            options(noreturn),
        );
    }
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const NULL: Self = Self {
        offset_low: 0,
        selector: 0,
        ist: 0,
        type_attr: 0,
        offset_mid: 0,
        offset_high: 0,
        reserved: 0,
    };
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::NULL; 256];

unsafe fn idt_set_gate(vector: usize, handler: u64) {
    IDT[vector] = IdtEntry {
        offset_low: handler as u16,
        selector: 0x08,
        ist: 0,
        type_attr: 0x8E,
        offset_mid: (handler >> 16) as u16,
        offset_high: (handler >> 32) as u32,
        reserved: 0,
    };
}

pub(crate) unsafe fn idt_init() {
    extern "C" {
        fn isr_stub_0();
        fn isr_stub_3();
        fn isr_stub_8();
        fn isr_stub_13();
        fn isr_stub_14();
        fn isr_stub_32();
        fn isr_stub_33();
        #[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
        fn isr_stub_64();
        #[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
        fn isr_stub_65();
        fn isr_stub_128();
        fn isr_stub_240();
    }

    idt_set_gate(0, isr_stub_0 as *const () as u64);
    idt_set_gate(3, isr_stub_3 as *const () as u64);
    idt_set_gate(8, isr_stub_8 as *const () as u64);
    idt_set_gate(13, isr_stub_13 as *const () as u64);
    idt_set_gate(14, isr_stub_14 as *const () as u64);
    idt_set_gate(32, isr_stub_32 as *const () as u64);
    idt_set_gate(33, isr_stub_33 as *const () as u64);
    #[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
    {
        idt_set_gate(64, isr_stub_64 as *const () as u64);
        idt_set_gate(65, isr_stub_65 as *const () as u64);
    }

    let handler = isr_stub_128 as *const () as u64;
    IDT[128] = IdtEntry {
        offset_low: handler as u16,
        selector: 0x08,
        ist: 0,
        type_attr: 0xEE,
        offset_mid: (handler >> 16) as u16,
        offset_high: (handler >> 32) as u32,
        reserved: 0,
    };

    // SMP IPI vector (full-os guide Part I.3): an interrupt gate (DPL=0).
    // Installed in every lane so APs can take the IPI on the base lane too.
    idt_set_gate(240, isr_stub_240 as *const () as u64);

    load_idt();
}

/// Load the shared IDT on the current CPU. The BSP calls this from idt_init;
/// each AP calls it (after loading the kernel GDT) so it can take the IPI.
pub(crate) unsafe fn load_idt() {
    let ptr = DtPtr {
        limit: (256 * core::mem::size_of::<IdtEntry>() - 1) as u16,
        base: IDT.as_ptr() as u64,
    };
    core::arch::asm!("lidt [{}]", in(reg) &ptr, options(nostack));
}
