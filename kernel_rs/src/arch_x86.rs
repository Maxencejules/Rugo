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

// Segment descriptors at indices 0..5 (null, kernel code, kernel data, user
// data, user code). Indices 5.. hold up to MAX_TSS 16-byte (two-slot) TSS
// descriptors, one per CPU that runs ring-3 tasks: CPU `c`'s descriptor is at
// GDT[5 + 2*c], selector 0x28 + 0x10*c. Per-CPU TSS descriptors are what let an
// application processor take a ring-3→ring-0 transition onto its OWN kernel
// stack (full-os guide Part I.3, SMP scheduler capstone). The BSP is slot 0
// (selector 0x28), unchanged from the single-TSS layout.
pub(crate) const MAX_TSS: usize = 8;
static mut GDT: [u64; 5 + 2 * MAX_TSS] = {
    let mut g = [0u64; 5 + 2 * MAX_TSS];
    g[0] = 0x0000_0000_0000_0000;
    g[1] = 0x00AF_9A00_0000_FFFF;
    g[2] = 0x00CF_9200_0000_FFFF;
    g[3] = 0x00CF_F200_0000_FFFF;
    g[4] = 0x00AF_FA00_0000_FFFF;
    g
};

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
    #[derive(Clone, Copy)]
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

    const TSS_EMPTY: Tss = Tss {
        reserved0: 0,
        rsp0: 0, rsp1: 0, rsp2: 0,
        reserved1: 0,
        ist: [0; 7],
        reserved2: 0,
        reserved3: 0,
        iopb_offset: 104,
    };

    // One TSS per CPU that can run ring-3 code (full-os guide Part I.3): each
    // CPU's TSS carries its OWN rsp0, so two CPUs taking a ring-3→ring-0
    // transition land on disjoint kernel stacks (the prerequisite for running
    // user tasks on more than one core at once).
    static mut TSS: [Tss; MAX_TSS] = [TSS_EMPTY; MAX_TSS];

    /// Install CPU `slot`'s TSS (rsp0 = `kernel_stack_top`) into its GDT
    /// descriptor and load it (`ltr`). Slot 0 is the BSP (selector 0x28); an
    /// AP passes its per-CPU index. The GDT word pair for distinct slots never
    /// overlaps, so concurrent CPUs each touch only their own descriptor.
    pub(crate) unsafe fn tss_init_cpu(slot: usize, kernel_stack_top: u64) {
        if slot >= MAX_TSS {
            return;
        }
        TSS[slot].rsp0 = kernel_stack_top;
        let tss_addr = core::ptr::addr_of!(TSS[slot]) as u64;
        let d = 5 + 2 * slot;
        GDT[d] = (103u64)
                | ((tss_addr & 0xFFFF) << 16)
                | (((tss_addr >> 16) & 0xFF) << 32)
                | (0x89u64 << 40)
                | (((tss_addr >> 24) & 0xFF) << 56);
        GDT[d + 1] = tss_addr >> 32;
        let limit = (core::mem::size_of_val(&GDT) - 1) as u16;
        let base = GDT.as_ptr() as u64;
        let gdt_ptr = DtPtr { limit, base };
        core::arch::asm!("lgdt [{}]", in(reg) &gdt_ptr);
        let sel = (d * 8) as u16; // 0x28 + 0x10*slot
        core::arch::asm!("ltr {sel:x}", sel = in(reg) sel, options(nostack));
    }

    /// BSP convenience: install + load the boot CPU's TSS (slot 0, selector
    /// 0x28). Preserves the original single-TSS call site/behaviour.
    pub(crate) unsafe fn tss_init(kernel_stack_top: u64) {
        tss_init_cpu(0, kernel_stack_top);
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

    /// Enter ring 3 at `code_va` with `user_sp`, passing `arg` in RDI (the
    /// SysV first-argument register). IF clear (not preemptible), so the brief
    /// user task runs without a timer landing mid-flight. Used by the SMP
    /// capstone to launch a ring-3 task on an application processor.
    #[cfg(feature = "go_test")]
    pub(crate) unsafe fn enter_ring3_with_arg(code_va: u64, user_sp: u64, arg: u64) -> ! {
        core::arch::asm!(
            "push 0x1B",
            "push {stack}",
            "push 0x002",
            "push 0x23",
            "push {code}",
            "mov rdi, {arg}",
            "iretq",
            stack = in(reg) user_sp,
            code = in(reg) code_va,
            arg = in(reg) arg,
            options(noreturn),
        );
    }

    // Per-CPU kernel stacks for application processors that run ring-3 tasks
    // (SMP capstone). Each AP's TSS rsp0 points at the top of its own stack, so
    // a ring-3→ring-0 transition (syscall/interrupt) on one AP never collides
    // with another CPU's kernel stack. Slot 0 is the BSP (it uses the boot
    // `stack_top`); AP slots 1..MAX_TSS use these.
    #[cfg(feature = "go_test")]
    const AP_KSTACK_SIZE: usize = 16384;
    #[cfg(feature = "go_test")]
    #[repr(align(16))]
    struct ApKstack([u8; AP_KSTACK_SIZE]);
    #[cfg(feature = "go_test")]
    static mut AP_KSTACK: [ApKstack; MAX_TSS] =
        [const { ApKstack([0u8; AP_KSTACK_SIZE]) }; MAX_TSS];

    /// Top (highest address, 16-aligned) of AP `slot`'s kernel stack.
    #[cfg(feature = "go_test")]
    pub(crate) unsafe fn ap_kstack_top(slot: usize) -> u64 {
        let base = core::ptr::addr_of!(AP_KSTACK[slot]) as u64;
        (base + AP_KSTACK_SIZE as u64) & !0xF
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
        // isr_stub_65 doubles as the LAPIC spurious-interrupt vector (set by
        // smp::x2apic_enable in every SMP lane), so its gate must exist
        // unconditionally — not only in the native/test lanes.
        fn isr_stub_65();
        fn isr_stub_128();
        // Vector 0x81: AP user-task report gate (SMP capstone, go lane only).
        #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
        fn isr_stub_129();
        fn isr_stub_240();
        fn isr_stub_241();
        fn isr_stub_242();
    }

    idt_set_gate(0, isr_stub_0 as *const () as u64);
    idt_set_gate(3, isr_stub_3 as *const () as u64);
    idt_set_gate(8, isr_stub_8 as *const () as u64);
    idt_set_gate(13, isr_stub_13 as *const () as u64);
    idt_set_gate(14, isr_stub_14 as *const () as u64);
    idt_set_gate(32, isr_stub_32 as *const () as u64);
    idt_set_gate(33, isr_stub_33 as *const () as u64);
    #[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
    idt_set_gate(64, isr_stub_64 as *const () as u64);
    // Vector 65 is the LAPIC spurious-interrupt sink (smp::x2apic_enable points
    // SVR here on every CPU that software-enables the APIC), so its gate must be
    // present in every lane — including the base `os.iso` the `-smp 4` test boots.
    // A spurious delivery to a not-present gate would #NP→#DF→teardown an AP.
    idt_set_gate(65, isr_stub_65 as *const () as u64);

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

    // Vector 0x81 (SMP capstone): a DPL=3 gate the ring-3 task an application
    // processor runs uses to report its result back to the kernel. Go lane
    // only — no other lane raises int 0x81, and gating it keeps the base/M3
    // lanes byte-for-byte unchanged.
    #[cfg(all(feature = "go_test", not(feature = "compat_real_test")))]
    {
        let h = isr_stub_129 as *const () as u64;
        IDT[129] = IdtEntry {
            offset_low: h as u16,
            selector: 0x08,
            ist: 0,
            type_attr: 0xEE,
            offset_mid: (h >> 16) as u16,
            offset_high: (h >> 32) as u32,
            reserved: 0,
        };
    }

    // SMP IPI + per-CPU LAPIC timer + TLB-shootdown vectors (full-os guide
    // Part I.3): interrupt gates (DPL=0). Installed in every lane so APs can
    // take them on any lane.
    idt_set_gate(240, isr_stub_240 as *const () as u64);
    idt_set_gate(241, isr_stub_241 as *const () as u64);
    idt_set_gate(242, isr_stub_242 as *const () as u64);

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
