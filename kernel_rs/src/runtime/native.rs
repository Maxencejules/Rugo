#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
use core::ptr::{read_volatile, write_volatile};

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
use crate::{
    pci_read32, pci_write32, serial_write, serial_write_u64_dec, BLK_DATA_PAGE, PciBdf,
};

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const IA32_APIC_BASE_MSR: u32 = 0x1B;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const IA32_APIC_BASE_X2APIC: u64 = 1 << 10;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const IA32_APIC_BASE_ENABLE: u64 = 1 << 11;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const X2APIC_MSR_BASE: u32 = 0x800;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const LAPIC_REG_ID: usize = 0x20;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const LAPIC_REG_EOI: usize = 0xB0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const LAPIC_REG_SVR: usize = 0xF0;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const PCI_CAP_ID_MSI: u8 = 0x05;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const PCI_CAP_ID_MSIX: u8 = 0x11;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_CLASS: u8 = 0x01;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_SUBCLASS: u8 = 0x08;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_PROGIF: u8 = 0x02;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_REG_CAP: usize = 0x0000;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_REG_CC: usize = 0x0014;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_REG_CSTS: usize = 0x001C;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_REG_AQA: usize = 0x0024;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_REG_ASQ: usize = 0x0028;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_REG_ACQ: usize = 0x0030;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_REG_DOORBELL_BASE: usize = 0x1000;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_CSTS_RDY: u32 = 1;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_CC_ENABLE: u32 = 1;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_CC_IOSQES_SHIFT: u32 = 16;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_CC_IOCQES_SHIFT: u32 = 20;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_OPC_FLUSH: u8 = 0x00;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_OPC_WRITE: u8 = 0x01;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_OPC_READ: u8 = 0x02;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_ADMIN_OPC_CREATE_IO_SQ: u8 = 0x01;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_ADMIN_OPC_CREATE_IO_CQ: u8 = 0x05;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_ADMIN_OPC_IDENTIFY: u8 = 0x06;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_IDENTIFY_CNS_NAMESPACE: u32 = 0x00;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_IDENTIFY_CNS_CONTROLLER: u32 = 0x01;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
pub const NATIVE_IRQ_VECTOR: usize = 64;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
pub const NATIVE_SPURIOUS_VECTOR: usize = 65;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_ADMIN_QID: u16 = 0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_IO_QID: u16 = 1;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_ADMIN_QUEUE_DEPTH: u16 = 16;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_IO_QUEUE_DEPTH: u16 = 16;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NVME_TIMEOUT_LOOPS: u32 = 20_000_000;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const PAGE_SIZE: u64 = 4096;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const PTE_PRESENT: u64 = 1;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const PTE_WRITE: u64 = 1 << 1;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const PTE_PWT: u64 = 1 << 3;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const PTE_PCD: u64 = 1 << 4;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NATIVE_MMIO_WINDOW_BASE: u64 = 0xFFFF_9000_0000_0000;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const NATIVE_MMIO_WINDOW_BYTES: usize = 2 * 1024 * 1024;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
#[repr(C, align(4096))]
struct NativePage([u8; 4096]);

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
#[repr(C)]
#[derive(Clone, Copy)]
struct NvmeCommand {
    opcode: u8,
    flags: u8,
    cid: u16,
    nsid: u32,
    rsvd2: u64,
    mptr: u64,
    prp1: u64,
    prp2: u64,
    cdw10: u32,
    cdw11: u32,
    cdw12: u32,
    cdw13: u32,
    cdw14: u32,
    cdw15: u32,
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
impl NvmeCommand {
    const fn empty() -> Self {
        Self {
            opcode: 0,
            flags: 0,
            cid: 0,
            nsid: 0,
            rsvd2: 0,
            mptr: 0,
            prp1: 0,
            prp2: 0,
            cdw10: 0,
            cdw11: 0,
            cdw12: 0,
            cdw13: 0,
            cdw14: 0,
            cdw15: 0,
        }
    }
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
#[repr(C)]
#[derive(Clone, Copy)]
struct NvmeCompletion {
    result: u32,
    rsvd: u32,
    sq_head: u16,
    sq_id: u16,
    cid: u16,
    status: u16,
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IrqMode {
    None,
    Msi,
    Msix,
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProbeError {
    NotFound,
    MmioBarMissing,
    IrqUnavailable,
    ControllerTimeout,
    IoQueueFailed,
    IdentifyFailed,
    NamespaceMissing,
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
#[derive(Clone, Copy)]
pub struct NvmeInfo {
    pub nsid: u32,
    pub block_bytes: u32,
    pub block_count: u64,
    pub admin_q_depth: u16,
    pub io_q_depth: u16,
    pub irq_mode: IrqMode,
    pub irq_vector: u8,
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_ADMIN_SQ: NativePage = NativePage([0; 4096]);
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_ADMIN_CQ: NativePage = NativePage([0; 4096]);
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_IO_SQ: NativePage = NativePage([0; 4096]);
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_IO_CQ: NativePage = NativePage([0; 4096]);
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_IDENTIFY_PAGE: NativePage = NativePage([0; 4096]);
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NATIVE_MMIO_PDPT: NativePage = NativePage([0; 4096]);
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NATIVE_MMIO_PD: NativePage = NativePage([0; 4096]);
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NATIVE_MMIO_PT: NativePage = NativePage([0; 4096]);

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NATIVE_KV2P_DELTA: u64 = 0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NATIVE_HHDM_OFFSET: u64 = 0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut LAPIC_READY: bool = false;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut LAPIC_X2_MODE: bool = false;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NATIVE_IRQ_COUNT: u64 = 0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NATIVE_SPURIOUS_COUNT: u64 = 0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_MMIO: *mut u8 = core::ptr::null_mut();
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_DOORBELL_STRIDE: usize = 4;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_PRESENT: bool = false;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_INFO: NvmeInfo = NvmeInfo {
    nsid: 0,
    block_bytes: 0,
    block_count: 0,
    admin_q_depth: 0,
    io_q_depth: 0,
    irq_mode: IrqMode::None,
    irq_vector: NATIVE_IRQ_VECTOR as u8,
};
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_ADMIN_SQ_TAIL: u16 = 0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_ADMIN_CQ_HEAD: u16 = 0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_ADMIN_CQ_PHASE: u16 = 1;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_IO_SQ_TAIL: u16 = 0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_IO_CQ_HEAD: u16 = 0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_IO_CQ_PHASE: u16 = 1;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_NEXT_CID: u16 = 1;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_RESET_COUNT: u32 = 0;
#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
static mut NVME_TIMEOUT_COUNT: u32 = 0;

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn rdmsr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nostack, nomem),
    );
    ((high as u64) << 32) | (low as u64)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn wrmsr(msr: u32, value: u64) {
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") value as u32,
        in("edx") (value >> 32) as u32,
        options(nostack, nomem),
    );
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    let regs = core::arch::x86_64::__cpuid_count(leaf, subleaf);
    (regs.eax, regs.ebx, regs.ecx, regs.edx)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn lapic_has_x2apic() -> bool {
    let (_, _, ecx, _) = cpuid(1, 0);
    (ecx & (1 << 21)) != 0
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn read_cr3() -> u64 {
    let cr3: u64;
    core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, nomem));
    cr3
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn native_phys(ptr: *const u8) -> u64 {
    (ptr as u64).wrapping_add(NATIVE_KV2P_DELTA)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn lapic_ptr() -> *mut u32 {
    let apic_base = (rdmsr(IA32_APIC_BASE_MSR) & 0xFFFF_F000) + NATIVE_HHDM_OFFSET;
    apic_base as *mut u32
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
const fn lapic_msr(offset: usize) -> u32 {
    X2APIC_MSR_BASE + ((offset >> 4) as u32)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn lapic_read(offset: usize) -> u32 {
    if LAPIC_X2_MODE {
        rdmsr(lapic_msr(offset)) as u32
    } else {
        read_volatile((lapic_ptr() as *mut u8).add(offset) as *mut u32)
    }
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn lapic_write(offset: usize, value: u32) {
    if LAPIC_X2_MODE {
        wrmsr(lapic_msr(offset), value as u64);
    } else {
        write_volatile((lapic_ptr() as *mut u8).add(offset) as *mut u32, value);
        let _ = lapic_read(offset);
    }
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn lapic_id() -> u8 {
    (lapic_read(LAPIC_REG_ID) >> 24) as u8
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn lapic_init() -> bool {
    if !lapic_has_x2apic() {
        return false;
    }
    let mut base = rdmsr(IA32_APIC_BASE_MSR);
    base |= IA32_APIC_BASE_ENABLE | IA32_APIC_BASE_X2APIC;
    wrmsr(IA32_APIC_BASE_MSR, base);
    LAPIC_X2_MODE = true;
    lapic_write(LAPIC_REG_SVR, (NATIVE_SPURIOUS_VECTOR as u32) | 0x100);
    LAPIC_READY = true;
    true
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn lapic_eoi() {
    if LAPIC_READY {
        lapic_write(LAPIC_REG_EOI, 0);
    }
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn pci_read16(bdf: PciBdf, offset: u8) -> u16 {
    let value = pci_read32(bdf.bus, bdf.dev, bdf.func, offset & !3);
    let shift = ((offset & 2) * 8) as u32;
    ((value >> shift) & 0xFFFF) as u16
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn pci_write16(bdf: PciBdf, offset: u8, value: u16) {
    let aligned = offset & !3;
    let shift = ((offset & 2) * 8) as u32;
    let mut reg = pci_read32(bdf.bus, bdf.dev, bdf.func, aligned);
    reg &= !(0xFFFFu32 << shift);
    reg |= (value as u32) << shift;
    pci_write32(bdf.bus, bdf.dev, bdf.func, aligned, reg);
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn pci_read8(bdf: PciBdf, offset: u8) -> u8 {
    let value = pci_read32(bdf.bus, bdf.dev, bdf.func, offset & !3);
    let shift = ((offset & 3) * 8) as u32;
    ((value >> shift) & 0xFF) as u8
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn pci_find_nvme() -> Option<PciBdf> {
    let mut dev = 0u8;
    while dev < 32 {
        let id = pci_read32(0, dev, 0, 0x00);
        if (id & 0xFFFF) != 0xFFFF {
            let class_reg = pci_read32(0, dev, 0, 0x08);
            let class = (class_reg >> 24) as u8;
            let subclass = (class_reg >> 16) as u8;
            let prog_if = (class_reg >> 8) as u8;
            if class == NVME_CLASS && subclass == NVME_SUBCLASS && prog_if == NVME_PROGIF {
                return Some(PciBdf { bus: 0, dev, func: 0 });
            }
        }
        dev += 1;
    }
    None
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn pci_enable_mem_bus_master(bdf: PciBdf) {
    let mut cmd = pci_read16(bdf, 0x04);
    cmd |= 0x0006;
    cmd &= !0x0400;
    pci_write16(bdf, 0x04, cmd);
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn pci_find_capability(bdf: PciBdf, cap_id: u8) -> Option<u8> {
    let status = pci_read16(bdf, 0x06);
    if status & (1 << 4) == 0 {
        return None;
    }
    let mut ptr = pci_read8(bdf, 0x34) & !0x3;
    let mut limit = 0u8;
    while ptr >= 0x40 && limit < 32 {
        if pci_read8(bdf, ptr) == cap_id {
            return Some(ptr);
        }
        ptr = pci_read8(bdf, ptr + 1) & !0x3;
        limit = limit.wrapping_add(1);
    }
    None
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn pci_bar_mmio_base(bdf: PciBdf, index: u8) -> Option<u64> {
    let bar_offset = 0x10u8.wrapping_add(index.wrapping_mul(4));
    let low = pci_read32(bdf.bus, bdf.dev, bdf.func, bar_offset);
    if low & 1 != 0 {
        return None;
    }
    let bar_type = (low >> 1) & 0x3;
    let mut base = (low & !0xFu32) as u64;
    if bar_type == 0x2 {
        let high = pci_read32(bdf.bus, bdf.dev, bdf.func, bar_offset.wrapping_add(4));
        base |= (high as u64) << 32;
    }
    if base == 0 {
        return None;
    }
    Some(base)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn native_dma_phys(ptr: *const u8, len: usize, kind: &[u8]) -> Option<u64> {
    if ptr.is_null() || len == 0 || len > 4096 {
        return None;
    }
    serial_write(b"DMA: map ok kind=");
    serial_write(kind);
    serial_write(b" bytes=");
    serial_write_u64_dec(len as u64);
    serial_write(b"\n");
    Some(native_phys(ptr))
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn irq_mode_bytes(mode: IrqMode) -> &'static [u8] {
    match mode {
        IrqMode::None => b"none",
        IrqMode::Msi => b"msi",
        IrqMode::Msix => b"msix",
    }
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn native_mmio_install_tables() -> bool {
    let cr3 = read_cr3();
    let pml4_phys = cr3 & 0x000F_FFFF_FFFF_F000;
    let pml4 = (pml4_phys + NATIVE_HHDM_OFFSET) as *mut u64;
    if pml4.is_null() {
        return false;
    }

    let pml4_idx = ((NATIVE_MMIO_WINDOW_BASE >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((NATIVE_MMIO_WINDOW_BASE >> 30) & 0x1FF) as usize;
    let pd_idx = ((NATIVE_MMIO_WINDOW_BASE >> 21) & 0x1FF) as usize;
    let table_flags = PTE_PRESENT | PTE_WRITE;

    core::ptr::write_bytes(NATIVE_MMIO_PDPT.0.as_mut_ptr(), 0, NATIVE_MMIO_PDPT.0.len());
    core::ptr::write_bytes(NATIVE_MMIO_PD.0.as_mut_ptr(), 0, NATIVE_MMIO_PD.0.len());
    let pdpt_phys = native_phys(NATIVE_MMIO_PDPT.0.as_ptr());
    let pd_phys = native_phys(NATIVE_MMIO_PD.0.as_ptr());
    let pt_phys = native_phys(NATIVE_MMIO_PT.0.as_ptr());

    write_volatile(pml4.add(pml4_idx), pdpt_phys | table_flags);
    write_volatile((NATIVE_MMIO_PDPT.0.as_mut_ptr() as *mut u64).add(pdpt_idx), pd_phys | table_flags);
    write_volatile((NATIVE_MMIO_PD.0.as_mut_ptr() as *mut u64).add(pd_idx), pt_phys | table_flags);
    true
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn native_mmio_map(phys: u64, len: usize) -> Option<*mut u8> {
    if len == 0 || len > NATIVE_MMIO_WINDOW_BYTES {
        return None;
    }
    if !native_mmio_install_tables() {
        return None;
    }

    let aligned = phys & !(PAGE_SIZE - 1);
    let offset = (phys - aligned) as usize;
    let page_count = (offset + len + (PAGE_SIZE as usize - 1)) / PAGE_SIZE as usize;
    if page_count > 512 {
        return None;
    }

    let pt = NATIVE_MMIO_PT.0.as_mut_ptr() as *mut u64;
    let page_flags = PTE_PRESENT | PTE_WRITE | PTE_PWT | PTE_PCD;
    let mut idx = 0usize;
    while idx < 512 {
        let entry = if idx < page_count {
            aligned.wrapping_add((idx as u64) * PAGE_SIZE) | page_flags
        } else {
            0
        };
        write_volatile(pt.add(idx), entry);
        idx += 1;
    }

    let mut page = 0usize;
    while page < page_count {
        let va = NATIVE_MMIO_WINDOW_BASE + (page as u64) * PAGE_SIZE;
        core::arch::asm!("invlpg [{}]", in(reg) va, options(nostack, preserves_flags));
        page += 1;
    }

    Some((NATIVE_MMIO_WINDOW_BASE + offset as u64) as *mut u8)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn enable_msi(bdf: PciBdf) -> bool {
    let cap = match pci_find_capability(bdf, PCI_CAP_ID_MSI) {
        Some(value) => value,
        None => return false,
    };
    let control = pci_read16(bdf, cap + 2);
    let is_64 = (control & (1 << 7)) != 0;
    let address = 0xFEE0_0000u32 | ((lapic_id() as u32) << 12);
    pci_write32(bdf.bus, bdf.dev, bdf.func, cap + 4, address);
    if is_64 {
        pci_write32(bdf.bus, bdf.dev, bdf.func, cap + 8, 0);
        pci_write16(bdf, cap + 12, NATIVE_IRQ_VECTOR as u16);
    } else {
        pci_write16(bdf, cap + 8, NATIVE_IRQ_VECTOR as u16);
    }
    pci_write16(bdf, cap + 2, control | 1);
    true
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn enable_msix(bdf: PciBdf) -> bool {
    let cap = match pci_find_capability(bdf, PCI_CAP_ID_MSIX) {
        Some(value) => value,
        None => return false,
    };
    let control = pci_read16(bdf, cap + 2);
    let table = pci_read32(bdf.bus, bdf.dev, bdf.func, cap + 4);
    let bir = (table & 0x7) as u8;
    let offset = (table & !0x7) as usize;
    let bar_base = match pci_bar_mmio_base(bdf, bir) {
        Some(value) => value,
        None => return false,
    };
    let table_ptr = match native_mmio_map(bar_base, offset + 16) {
        Some(value) => value,
        None => return false,
    };
    let entry = table_ptr.add(offset);
    write_volatile(entry.add(12) as *mut u32, 1);
    write_volatile(entry as *mut u32, 0xFEE0_0000u32 | ((lapic_id() as u32) << 12));
    write_volatile(entry.add(4) as *mut u32, 0);
    write_volatile(entry.add(8) as *mut u32, NATIVE_IRQ_VECTOR as u32);
    write_volatile(entry.add(12) as *mut u32, 0);
    pci_write16(bdf, cap + 2, (control | (1 << 15)) & !(1 << 14));
    true
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn bind_irq(bdf: PciBdf) -> IrqMode {
    if !lapic_init() {
        return IrqMode::None;
    }
    let mode = if enable_msi(bdf) {
        IrqMode::Msi
    } else if enable_msix(bdf) {
        IrqMode::Msix
    } else {
        IrqMode::None
    };
    if mode != IrqMode::None {
        serial_write(b"IRQ: vector bound vec=");
        serial_write_u64_dec(NATIVE_IRQ_VECTOR as u64);
        serial_write(b" mode=");
        serial_write(irq_mode_bytes(mode));
        serial_write(b"\n");
    }
    mode
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_mmio_read32(offset: usize) -> u32 {
    read_volatile(NVME_MMIO.add(offset) as *const u32)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_mmio_write32(offset: usize, value: u32) {
    write_volatile(NVME_MMIO.add(offset) as *mut u32, value);
    let _ = nvme_mmio_read32(offset);
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_mmio_read64(offset: usize) -> u64 {
    read_volatile(NVME_MMIO.add(offset) as *const u64)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_mmio_write64(offset: usize, value: u64) {
    write_volatile(NVME_MMIO.add(offset) as *mut u64, value);
    let _ = nvme_mmio_read32(offset);
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_sq_doorbell(qid: u16) -> usize {
    NVME_REG_DOORBELL_BASE + ((2 * qid as usize) * NVME_DOORBELL_STRIDE)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_cq_doorbell(qid: u16) -> usize {
    NVME_REG_DOORBELL_BASE + (((2 * qid as usize) + 1) * NVME_DOORBELL_STRIDE)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_wait_ready(expect_ready: bool) -> bool {
    let mut timeout = NVME_TIMEOUT_LOOPS;
    while timeout != 0 {
        let ready = (nvme_mmio_read32(NVME_REG_CSTS) & NVME_CSTS_RDY) != 0;
        if ready == expect_ready {
            return true;
        }
        core::arch::asm!("pause", options(nomem, nostack));
        timeout -= 1;
    }
    NVME_TIMEOUT_COUNT = NVME_TIMEOUT_COUNT.wrapping_add(1);
    false
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_reset_queues() {
    core::ptr::write_bytes(NVME_ADMIN_SQ.0.as_mut_ptr(), 0, NVME_ADMIN_SQ.0.len());
    core::ptr::write_bytes(NVME_ADMIN_CQ.0.as_mut_ptr(), 0, NVME_ADMIN_CQ.0.len());
    core::ptr::write_bytes(NVME_IO_SQ.0.as_mut_ptr(), 0, NVME_IO_SQ.0.len());
    core::ptr::write_bytes(NVME_IO_CQ.0.as_mut_ptr(), 0, NVME_IO_CQ.0.len());
    NVME_ADMIN_SQ_TAIL = 0;
    NVME_ADMIN_CQ_HEAD = 0;
    NVME_ADMIN_CQ_PHASE = 1;
    NVME_IO_SQ_TAIL = 0;
    NVME_IO_CQ_HEAD = 0;
    NVME_IO_CQ_PHASE = 1;
    NVME_NEXT_CID = 1;
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_enable_controller() -> bool {
    nvme_mmio_write32(NVME_REG_CC, 0);
    if !nvme_wait_ready(false) {
        return false;
    }
    nvme_reset_queues();
    let asq = match native_dma_phys(NVME_ADMIN_SQ.0.as_ptr(), 4096, b"admin_sq") {
        Some(value) => value,
        None => return false,
    };
    let acq = match native_dma_phys(NVME_ADMIN_CQ.0.as_ptr(), 4096, b"admin_cq") {
        Some(value) => value,
        None => return false,
    };
    let aqa =
        ((NVME_ADMIN_QUEUE_DEPTH - 1) as u32) | (((NVME_ADMIN_QUEUE_DEPTH - 1) as u32) << 16);
    nvme_mmio_write32(NVME_REG_AQA, aqa);
    nvme_mmio_write64(NVME_REG_ASQ, asq);
    nvme_mmio_write64(NVME_REG_ACQ, acq);
    let cc = NVME_CC_ENABLE | (6 << NVME_CC_IOSQES_SHIFT) | (4 << NVME_CC_IOCQES_SHIFT);
    nvme_mmio_write32(NVME_REG_CC, cc);
    if !nvme_wait_ready(true) {
        return false;
    }
    NVME_RESET_COUNT = NVME_RESET_COUNT.wrapping_add(1);
    true
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_submit_command(admin: bool, mut command: NvmeCommand) -> Option<u32> {
    let cid = NVME_NEXT_CID;
    NVME_NEXT_CID = NVME_NEXT_CID.wrapping_add(1);
    command.cid = cid;

    let (sq_page, sq_tail, sq_depth, sq_qid) = if admin {
        (
            NVME_ADMIN_SQ.0.as_mut_ptr() as *mut NvmeCommand,
            &mut NVME_ADMIN_SQ_TAIL,
            NVME_ADMIN_QUEUE_DEPTH,
            NVME_ADMIN_QID,
        )
    } else {
        (
            NVME_IO_SQ.0.as_mut_ptr() as *mut NvmeCommand,
            &mut NVME_IO_SQ_TAIL,
            NVME_IO_QUEUE_DEPTH,
            NVME_IO_QID,
        )
    };

    write_volatile(sq_page.add(*sq_tail as usize), command);
    core::arch::asm!("mfence", options(nostack));
    *sq_tail = (*sq_tail + 1) % sq_depth;
    nvme_mmio_write32(nvme_sq_doorbell(sq_qid), *sq_tail as u32);

    let mut timeout = NVME_TIMEOUT_LOOPS;
    if NVME_INFO.irq_mode != IrqMode::None {
        core::arch::asm!("sti", options(nostack));
    }
    loop {
        let (cq_page, cq_head, cq_phase, cq_depth, cq_qid) = if admin {
            (
                NVME_ADMIN_CQ.0.as_ptr() as *const NvmeCompletion,
                &mut NVME_ADMIN_CQ_HEAD,
                &mut NVME_ADMIN_CQ_PHASE,
                NVME_ADMIN_QUEUE_DEPTH,
                NVME_ADMIN_QID,
            )
        } else {
            (
                NVME_IO_CQ.0.as_ptr() as *const NvmeCompletion,
                &mut NVME_IO_CQ_HEAD,
                &mut NVME_IO_CQ_PHASE,
                NVME_IO_QUEUE_DEPTH,
                NVME_IO_QID,
            )
        };
        let cqe = read_volatile(cq_page.add(*cq_head as usize));
        let phase = cqe.status & 1;
        if phase == *cq_phase && cqe.cid == cid {
            if NVME_INFO.irq_mode != IrqMode::None {
                core::arch::asm!("cli", options(nostack));
            }
            let status_code = cqe.status >> 1;
            *cq_head += 1;
            if *cq_head == cq_depth {
                *cq_head = 0;
                *cq_phase ^= 1;
            }
            nvme_mmio_write32(nvme_cq_doorbell(cq_qid), *cq_head as u32);
            if status_code != 0 {
                return None;
            }
            return Some(cqe.result);
        }
        if timeout == 0 {
            if NVME_INFO.irq_mode != IrqMode::None {
                core::arch::asm!("cli", options(nostack));
            }
            NVME_TIMEOUT_COUNT = NVME_TIMEOUT_COUNT.wrapping_add(1);
            serial_write(b"BLK: flush timeout\n");
            return None;
        }
        timeout -= 1;
        core::arch::asm!("pause", options(nomem, nostack));
    }
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_identify_controller() -> bool {
    core::ptr::write_bytes(NVME_IDENTIFY_PAGE.0.as_mut_ptr(), 0, NVME_IDENTIFY_PAGE.0.len());
    let identify_phys = match native_dma_phys(NVME_IDENTIFY_PAGE.0.as_ptr(), 4096, b"identify_ctrl")
    {
        Some(value) => value,
        None => return false,
    };
    let mut command = NvmeCommand::empty();
    command.opcode = NVME_ADMIN_OPC_IDENTIFY;
    command.prp1 = identify_phys;
    command.cdw10 = NVME_IDENTIFY_CNS_CONTROLLER;
    nvme_submit_command(true, command).is_some()
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_identify_namespace() -> bool {
    core::ptr::write_bytes(NVME_IDENTIFY_PAGE.0.as_mut_ptr(), 0, NVME_IDENTIFY_PAGE.0.len());
    let identify_phys = match native_dma_phys(NVME_IDENTIFY_PAGE.0.as_ptr(), 4096, b"identify_ns")
    {
        Some(value) => value,
        None => return false,
    };
    let mut command = NvmeCommand::empty();
    command.opcode = NVME_ADMIN_OPC_IDENTIFY;
    command.nsid = 1;
    command.prp1 = identify_phys;
    command.cdw10 = NVME_IDENTIFY_CNS_NAMESPACE;
    if nvme_submit_command(true, command).is_none() {
        return false;
    }
    let nsze = u64::from_le_bytes([
        NVME_IDENTIFY_PAGE.0[0],
        NVME_IDENTIFY_PAGE.0[1],
        NVME_IDENTIFY_PAGE.0[2],
        NVME_IDENTIFY_PAGE.0[3],
        NVME_IDENTIFY_PAGE.0[4],
        NVME_IDENTIFY_PAGE.0[5],
        NVME_IDENTIFY_PAGE.0[6],
        NVME_IDENTIFY_PAGE.0[7],
    ]);
    if nsze == 0 {
        return false;
    }
    let flbas = NVME_IDENTIFY_PAGE.0[26] & 0x0F;
    let lbaf = 128 + (flbas as usize) * 4;
    let lbads = NVME_IDENTIFY_PAGE.0[lbaf + 2];
    NVME_INFO.nsid = 1;
    NVME_INFO.block_count = nsze;
    NVME_INFO.block_bytes = 1u32 << lbads;
    NVME_INFO.admin_q_depth = NVME_ADMIN_QUEUE_DEPTH;
    NVME_INFO.io_q_depth = NVME_IO_QUEUE_DEPTH;
    true
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn nvme_create_io_queues() -> bool {
    let cq_phys = match native_dma_phys(NVME_IO_CQ.0.as_ptr(), 4096, b"io_cq") {
        Some(value) => value,
        None => return false,
    };
    let sq_phys = match native_dma_phys(NVME_IO_SQ.0.as_ptr(), 4096, b"io_sq") {
        Some(value) => value,
        None => return false,
    };

    let mut create_cq = NvmeCommand::empty();
    create_cq.opcode = NVME_ADMIN_OPC_CREATE_IO_CQ;
    create_cq.prp1 = cq_phys;
    create_cq.cdw10 = (NVME_IO_QID as u32) | (((NVME_IO_QUEUE_DEPTH - 1) as u32) << 16);
    create_cq.cdw11 = 1 | if NVME_INFO.irq_mode != IrqMode::None { 2 } else { 0 };
    if nvme_submit_command(true, create_cq).is_none() {
        return false;
    }

    let mut create_sq = NvmeCommand::empty();
    create_sq.opcode = NVME_ADMIN_OPC_CREATE_IO_SQ;
    create_sq.prp1 = sq_phys;
    create_sq.cdw10 = (NVME_IO_QID as u32) | (((NVME_IO_QUEUE_DEPTH - 1) as u32) << 16);
    create_sq.cdw11 = 1 | ((NVME_IO_QID as u32) << 16);
    nvme_submit_command(true, create_sq).is_some()
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn emit_nvme_identify() {
    serial_write(b"NVME: identify ok nsid=");
    serial_write_u64_dec(NVME_INFO.nsid as u64);
    serial_write(b" lba=");
    serial_write_u64_dec(NVME_INFO.block_bytes as u64);
    serial_write(b" blocks=");
    serial_write_u64_dec(NVME_INFO.block_count);
    serial_write(b"\n");
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn emit_nvme_ready() {
    serial_write(b"DRV: bind driver=nvme\n");
    serial_write(b"FW: allow signed driver=nvme\n");
    serial_write(b"NVME: ready\n");
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
unsafe fn emit_nvme_io_ready() {
    serial_write(b"NVME: io queue ok depth=");
    serial_write_u64_dec(NVME_IO_QUEUE_DEPTH as u64);
    serial_write(b" irq_hits=");
    serial_write_u64_dec(NATIVE_IRQ_COUNT);
    serial_write(b"\n");
    serial_write(b"NVME: reset recover count=");
    serial_write_u64_dec(NVME_RESET_COUNT as u64);
    serial_write(b"\n");
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
pub unsafe fn handle_irq(vector: u64) -> bool {
    match vector as usize {
        NATIVE_IRQ_VECTOR => {
            NATIVE_IRQ_COUNT = NATIVE_IRQ_COUNT.wrapping_add(1);
            lapic_eoi();
            true
        }
        NATIVE_SPURIOUS_VECTOR => {
            NATIVE_SPURIOUS_COUNT = NATIVE_SPURIOUS_COUNT.wrapping_add(1);
            lapic_eoi();
            true
        }
        _ => false,
    }
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
pub unsafe fn probe_nvme(kv2p_delta: u64, hhdm_offset: u64) -> Result<NvmeInfo, ProbeError> {
    let bdf = match pci_find_nvme() {
        Some(value) => value,
        None => return Err(ProbeError::NotFound),
    };
    let bar0 = match pci_bar_mmio_base(bdf, 0) {
        Some(value) => value,
        None => return Err(ProbeError::MmioBarMissing),
    };

    NATIVE_KV2P_DELTA = kv2p_delta;
    NATIVE_HHDM_OFFSET = hhdm_offset;
    pci_enable_mem_bus_master(bdf);
    NVME_INFO.irq_mode = bind_irq(bdf);
    if NVME_INFO.irq_mode == IrqMode::None {
        return Err(ProbeError::IrqUnavailable);
    }
    NVME_MMIO = match native_mmio_map(bar0, 0x10000) {
        Some(value) => value,
        None => return Err(ProbeError::MmioBarMissing),
    };
    serial_write(b"BAR: map ok bar=0 bytes=65536\n");
    emit_nvme_ready();
    let cap = nvme_mmio_read64(NVME_REG_CAP);
    NVME_DOORBELL_STRIDE = 4usize << (((cap >> 32) & 0xF) as usize);
    if !nvme_enable_controller() {
        return Err(ProbeError::ControllerTimeout);
    }
    if !nvme_identify_controller() {
        return Err(ProbeError::IdentifyFailed);
    }
    if !nvme_create_io_queues() {
        return Err(ProbeError::IoQueueFailed);
    }
    if !nvme_identify_namespace() {
        return Err(ProbeError::NamespaceMissing);
    }
    emit_nvme_identify();
    emit_nvme_io_ready();
    NVME_PRESENT = true;
    NVME_INFO.irq_vector = NATIVE_IRQ_VECTOR as u8;
    Ok(NVME_INFO)
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
pub unsafe fn nvme_available() -> bool {
    NVME_PRESENT
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
pub unsafe fn nvme_info() -> Option<NvmeInfo> {
    if NVME_PRESENT {
        Some(NVME_INFO)
    } else {
        None
    }
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
pub unsafe fn nvme_irq_hits() -> u64 {
    NATIVE_IRQ_COUNT
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
pub unsafe fn nvme_read_write(write: bool, sector512: u64, len: usize, fua: bool) -> bool {
    if !NVME_PRESENT || NVME_INFO.block_bytes == 0 || len == 0 {
        return false;
    }
    let byte_offset = sector512.wrapping_mul(512);
    let block_bytes = NVME_INFO.block_bytes as u64;
    if byte_offset % block_bytes != 0 || (len as u64) % block_bytes != 0 {
        return false;
    }
    let lba = byte_offset / block_bytes;
    let blocks = (len as u64) / block_bytes;
    if blocks == 0 || blocks > 8 {
        return false;
    }

    let data_phys = match native_dma_phys(BLK_DATA_PAGE.0.as_ptr(), 4096, b"blk_data") {
        Some(value) => value,
        None => return false,
    };
    let mut command = NvmeCommand::empty();
    command.opcode = if write { NVME_OPC_WRITE } else { NVME_OPC_READ };
    command.nsid = NVME_INFO.nsid;
    command.prp1 = data_phys;
    command.cdw10 = lba as u32;
    command.cdw11 = (lba >> 32) as u32;
    command.cdw12 = ((blocks - 1) as u32) | if fua { 1 << 30 } else { 0 };
    nvme_submit_command(false, command).is_some()
}

#[cfg(any(feature = "blk_test", feature = "fs_test", feature = "go_test"))]
pub unsafe fn nvme_flush() -> bool {
    if !NVME_PRESENT {
        return false;
    }
    let mut command = NvmeCommand::empty();
    command.opcode = NVME_OPC_FLUSH;
    command.nsid = NVME_INFO.nsid;
    nvme_submit_command(false, command).is_some()
}
